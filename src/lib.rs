use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::DeriveInput;

mod args;
mod general;
mod struct_info;

use args::Args;
use struct_info::StructInfo;


/// `#[ref_struct(cmd[0], .., cmd[n])]` <br/>
/// where `cmd[i]` can be one of next:
/// * `name(output_struct_name)`
/// * `clone(clone_field_1, .., clone_field_k)`
/// * `ignore(ignore_field_1, .., ignore_field_k)`
/// * `derive(derive_path_1, .., derive_path_k)`
/// * `public`
/// * `use_cow`
/// * | `ignore_struct(name(ignore_struct_name), derive(ignore_derive_path_1, .., ignore_derive_path_k))` <br/>
///   | (imply `use_cow`)
#[proc_macro_attribute]
pub fn ref_struct(args: TokenStream, item: TokenStream) -> TokenStream {
    let args: Args = syn::parse(args).unwrap();
    // todo!("args = {args:#?}");

    let input: DeriveInput = syn::parse(item).unwrap();
    let input_info = StructInfo::from_derive_input(&input);
    
    for (ignore_field, _) in &args.ignore {
        if !input_info.field_names_set.contains(ignore_field) {
            panic!("unknown [ignore] fields: `{ignore_field}`")
        }
    }
    for (clone_field, _) in &args.clone {
        if !input_info.field_names_set.contains(clone_field) {
            panic!("unknown [clone] fields: `{clone_field}`")
        }
    }

    let in_name = input_info.name;
    let out_name = args.name.unwrap_or(format_ident!("Ref{}", in_name));    

    let maybe_pub = if args.is_pub { quote! { pub } } else { quote::quote!{ } };
    let derives = args.derive;

    let mut ignore_fields = Vec::new();
    let mut ignore_tys = Vec::new();
    let mut clone_fields = Vec::new();
    let mut clone_tys = Vec::new();
    let mut ref_fields = Vec::new();
    let mut ref_tys = Vec::new();

    for (i, ident) in input_info.field_names.iter().enumerate() {
        let ty = input_info.field_types[i].clone();
        let field = ident.clone();

        let s = ident.to_string();
        
        if args.ignore.contains_key(&s) {
            ignore_tys.push(ty);
            ignore_fields.push(field);
        } else if args.clone.contains_key(&s) {
            clone_tys.push(ty);
            clone_fields.push(field);
        } else {
            ref_tys.push(ty);
            ref_fields.push(field);
        }
    }

    let ref_fields_define = if args.use_cow {
        quote!{ #(#ref_fields: std::borrow::Cow<'x, #ref_tys>,)* }
    } else {
        quote!{ #(#ref_fields: &'x #ref_tys,)* }
    };

    let ref_fields_init = if args.use_cow {
        quote!{ #(#ref_fields: std::borrow::Cow::Borrowed(&x.#ref_fields),)* }
    } else {
        quote!{ #(#ref_fields: &x.#ref_fields,)* }
    };

    let maybe_ignore = args.ignore_struct_args.map(|ignore_args|{
        let derives = ignore_args.derive;
        let ignore_name = ignore_args.name.unwrap_or(format_ident!("Ignore{}", in_name)); 
        
        let struct_define = quote! {
            #[derive( #(#derives,)* )]
            struct #ignore_name {
                #(#ignore_fields: #ignore_tys,)*
            }
        };

        let merge_fn = quote! {
            pub fn merge(self, ignored: #ignore_name) -> #in_name {
                #in_name {
                    #(#ref_fields: self.#ref_fields.into_owned(),)*
                    #(#clone_fields: self.#clone_fields,)*
                    #(#ignore_fields: ignored.#ignore_fields,)*
                }
            }
        };

        (struct_define, merge_fn)
    });

    let (maybe_ignore_struct, maybe_merge_fn) = maybe_ignore
        .map(|(define, merge_fn)|(Some(define), Some(merge_fn)))
        .unwrap_or_default();

    let cgen = quote!{        
        #input

        #maybe_ignore_struct

        #[derive( #(#derives,)* )]
        #maybe_pub struct #out_name<'x> {
            #ref_fields_define
            #(#clone_fields: #clone_tys,)*
        }

        impl<'x> #out_name<'x> {
            pub fn new(x: &'x #in_name) -> Self {
                Self {
                    #ref_fields_init
                    #(#clone_fields: x.#clone_fields.clone(),)*
                }
            }

            #maybe_merge_fn
        }
    };

    // todo!("\n```\n{}\n```\n", cgen.to_string());
    TokenStream::from(cgen)
}