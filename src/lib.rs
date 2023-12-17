use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse::Parser};

use proc_macro2::TokenStream as TokenStream2;


#[derive(Debug)]
struct IdentedMap {
    name: syn::Ident,
    map: HashMap<String, syn::Ident>,
}

impl syn::parse::Parse for IdentedMap {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<syn::Ident>()?;

        let content;
        let _ = syn::parenthesized!(content in input);
        let set = syn::punctuated::Punctuated::<syn::Ident, syn::Token![,]>::parse_terminated(&content)?;

        let mut map = HashMap::new();
        for ident in set {
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
struct Args {
    ignore: IdentedMap,
    clone: IdentedMap,
}
impl syn::parse::Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ignore = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let clone = input.parse()?;
        Ok(Self {
            ignore,
            clone,
        })
    }
}

struct StructInfo {
    name: syn::Ident,
    field_names: Vec<syn::Ident>,
    field_types: Vec<TokenStream2>,
}
impl StructInfo {
    fn from_derive_input(input: DeriveInput) -> Self {
        let name = input.ident;

        let mut struct_fields = if let syn::Data::Struct(ds) = input.data {
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
/// `#[ref_struct(ignore(ignore_field_1, .., ignore_field_n), clone(clone_field_1, .., clone_field_n))]`
#[proc_macro_attribute]
pub fn ref_struct(args: TokenStream, item: TokenStream) -> TokenStream {
    let args: Args = syn::parse(args).unwrap();

    let input: DeriveInput = syn::parse(item).unwrap();
    let input_info = StructInfo::from_derive_input(input);
    
    todo!("")
    // todo!("args = {args:?}\n\n\nstruct = {struct_field_types:?}")
}