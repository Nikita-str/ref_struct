use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::quote;
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
struct Args {
    name: Option<syn::Ident>,
    clone: HashMap<String, syn::Ident>,
    ignore: HashMap<String, syn::Ident>,
}
impl syn::parse::Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
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

        const IGNORE_CMD: &str = "ignore";
        const CLONE_CMD: &str = "clone";
        const NAME_CMD: &str = "name";
        const ALL_CMD: &[&str] = &[NAME_CMD, CLONE_CMD, IGNORE_CMD];

        let mut name = None;
        let mut ignore = None;
        let mut clone = None;

        loop {
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

            if input.is_empty() { break }
            input.parse::<syn::Token![,]>()?;
        }

        let ignore = ignore.unwrap_or_else(||HashMap::new());
        let clone = clone.unwrap_or_else(||HashMap::new());

        Ok(Self {
            name,
            clone,
            ignore,
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
/// `#[ref_struct(cmd[0], .., cmd[n])]` <br/>
/// where `cmd[i]` can be one of next:
/// * `name(output_struct_name)`
/// * `clone(clone_field_1, .., clone_field_k)`
/// * `ignore(ignore_field_1, .., ignore_field_m)`
#[proc_macro_attribute]
pub fn ref_struct(args: TokenStream, item: TokenStream) -> TokenStream {
    let args: Args = syn::parse(args).unwrap();

    let input: DeriveInput = syn::parse(item).unwrap();
    let input_info = StructInfo::from_derive_input(input);
    

    todo!("args = {args:#?}")
    // todo!("args = {args:?}\n\n\nstruct = {struct_field_types:?}")
}