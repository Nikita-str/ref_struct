use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::DeriveInput;
use syn::punctuated::Punctuated as Puncted;
use syn::parse::{Parse, ParseStream};

use proc_macro2::TokenStream as TokenStream2;


#[derive(Debug)]
struct IdentedMap {
    name: syn::Ident,
    map: HashMap<String, syn::Ident>,
}

impl Parse for IdentedMap {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<syn::Ident>()?;

        let content;
        let _ = syn::parenthesized!(content in input);
        let list = Puncted::<syn::Ident, syn::Token![,]>::parse_terminated(&content)?;

        let mut map = HashMap::new();
        for ident in list {
            let s = ident.to_string();
            map.insert(s, ident);
        }

        Ok(Self {
            name,
            map,
        })
    }
}

#[derive(Debug)]
struct IdentedList<T> {
    #[allow(unused)]
    name: syn::Ident,
    list: Vec<T>,
}

impl<T: Parse> Parse for IdentedList<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<syn::Ident>()?;

        let content;
        let _ = syn::parenthesized!(content in input);
        let list = Puncted::<T, syn::Token![,]>::parse_terminated(&content)?;
        let list = list.into_iter().collect();

        Ok(Self {
            name,
            list,
        })
    }
}

#[derive(Debug)]
struct Args {
    name: Option<syn::Ident>,
    clone: HashMap<String, syn::Ident>,
    ignore: HashMap<String, syn::Ident>,
    derive: Vec<TokenStream2>,
    is_pub: bool,
    use_cow: bool,
}
impl syn::parse::Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        const USE_COW: &str = "use_cow";
        const PUB_CMD: &str = "public";
        const DERIVE_CMD: &str = "derive";
        const IGNORE_CMD: &str = "ignore";
        const CLONE_CMD: &str = "clone";
        const NAME_CMD: &str = "name";
        const ALL_CMD: &[&str] = &[NAME_CMD, CLONE_CMD, IGNORE_CMD,  DERIVE_CMD, PUB_CMD, USE_COW];

        let mut use_cow = false;
        let mut is_pub = false;
        let mut name = None;
        let mut ignore = None;
        let mut clone = None;
        let mut derive = Vec::new();

        loop {
            // [+]   LOOP RELATED MACRO DEFINES
            macro_rules! cmd_done {
                () => {{
                    if input.is_empty() { break }
                    input.parse::<syn::Token![,]>()?;
                    continue
                }};
            }
            macro_rules! twiced_err {
                ($cmd: expr) => {{
                    let span = input.span();
                    let message = format!("twiced `{}(..)` cmd", $cmd);
                    return Err(syn::Error::new(span, message))      
                }};
                (? $x: ident => $cmd: expr) => {{
                    if $x.is_some() {
                        twiced_err!($cmd)
                    }
                }};
            }
            // [-]   LOOP RELATED MACRO DEFINES

            let input_checker = input.fork();
            // twice parse `syn::Ident` seems k ¯\_(ツ)_/¯
            let cmd = input_checker.parse::<syn::Ident>()?.to_string();
            if cmd == DERIVE_CMD {
                let path_list: IdentedList<syn::Path> = input.parse()?;
                let mut add_derive = path_list.list.into_iter().map(|path|quote!{ #path }).collect::<Vec<_>>();
                derive.append(&mut add_derive);
                cmd_done!()
            }
            if cmd == PUB_CMD {
                input.parse::<syn::Ident>()?;
                is_pub = true;
                cmd_done!()
            }
            if cmd == USE_COW {
                input.parse::<syn::Ident>()?;
                use_cow = true;
                cmd_done!()
            }

            let idented_map: IdentedMap = input.parse()?;
            let cmd = idented_map.name.to_string();
            match cmd.as_str() {
                IGNORE_CMD => {
                    twiced_err!(? ignore => IGNORE_CMD);
                    ignore = Some(idented_map.map);
                }
                CLONE_CMD => {
                    twiced_err!(? clone => CLONE_CMD);                    
                    clone = Some(idented_map.map);
                }
                NAME_CMD => {
                    twiced_err!(? name => NAME_CMD);
                    if idented_map.map.len() != 1 {
                        return Err(syn::Error::new(input.span(), "expected `name(output_struct_name)`"))
                    }
                    name = Some(idented_map.map.into_iter().next().unwrap().1)
                }
                _ => {
                    let span = input.span();
                    let message = format!("expected `cmd(..)` where cmd is one of {ALL_CMD:?}");
                    return Err(syn::Error::new(span, message))
                }
            }

            cmd_done!()
        }

        let ignore = ignore.unwrap_or_else(||HashMap::new());
        let clone = clone.unwrap_or_else(||HashMap::new());

        Ok(Self {
            name,
            clone,
            ignore,
            derive,
            is_pub,
            use_cow,
        })
    }
}

struct StructInfo {
    name: syn::Ident,
    field_names: Vec<syn::Ident>,
    field_types: Vec<TokenStream2>,
}
impl StructInfo {
    fn from_derive_input(input: &DeriveInput) -> Self {
        let name = input.ident.clone();

        let struct_fields = if let syn::Data::Struct(ds) = input.data.clone() {
            ds.fields.into_iter().map(|field|{
                if let Some(name) = field.ident {
                    (name, field.ty)
                } else {
                    unimplemented!("unnamed field case")
                }
            }).collect::<Vec<_>>()
        } else {
            unimplemented!("non struct case");
        };
    
        let field_names = struct_fields.iter()
            .map(|field|field.0.clone()).collect::<Vec<_>>();
    
        let field_types = struct_fields.iter()
            .map(|(_, ty)|quote!{ #ty }).collect::<Vec<_>>();

        Self {
            name,
            field_names,
            field_types,
        }
    }
}

/// # general form
/// `#[ref_struct(cmd[0], .., cmd[n])]` <br/>
/// where `cmd[i]` can be one of next:
/// * `name(output_struct_name)`
/// * `clone(clone_field_1, .., clone_field_k)`
/// * `ignore(ignore_field_1, .., ignore_field_k)`
/// * `derive(derive_path_1, .., derive_path_k)`
/// * `public`
/// * `use_cow`
#[proc_macro_attribute]
pub fn ref_struct(args: TokenStream, item: TokenStream) -> TokenStream {
    let args: Args = syn::parse(args).unwrap();

    let input: DeriveInput = syn::parse(item).unwrap();
    let input_info = StructInfo::from_derive_input(&input);
    
    let in_name = input_info.name;
    let out_name = args.name.unwrap_or(format_ident!("Ref{}", in_name));    

    let maybe_pub = if args.is_pub { quote! { pub } } else { quote::quote!{ } };
    let derives = args.derive;

    let mut clone_fields = Vec::new();
    let mut clone_tys = Vec::new();
    let mut ref_fields = Vec::new();
    let mut ref_tys = Vec::new();

    for (i, ident) in input_info.field_names.iter().enumerate() {
        let s = ident.to_string();
        if args.ignore.contains_key(&s) { continue }
        if args.clone.contains_key(&s) {
            clone_tys.push(input_info.field_types[i].clone());
            clone_fields.push(ident.clone());
        } else {
            ref_tys.push(input_info.field_types[i].clone());
            ref_fields.push(ident.clone());
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

    let cgen = quote!{
        #input

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
        }
    };

    // todo!("args = {args:#?}")
    // todo!("\n```\n{}\n```\n", cgen.to_string());
    TokenStream::from(cgen)
}