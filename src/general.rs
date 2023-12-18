use std::collections::HashMap;
use syn::parse::{Parse, ParseStream};

type CommaSeparated<T> = syn::punctuated::Punctuated::<T, syn::Token![,]>;


// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// [+] [Indents]
#[derive(Debug)]
pub struct UnnamedIndents {
    pub map: HashMap<String, syn::Ident>,
}
impl Parse for UnnamedIndents {
    fn parse(input: ParseStream) -> syn::Result<Self> {

        let content;
        let _ = syn::parenthesized!(content in input);
        let list = CommaSeparated::<syn::Ident>::parse_terminated(&content)?;

        let mut map = HashMap::new();
        for ident in list {
            let s = ident.to_string();
            map.insert(s, ident);
        }

        Ok(Self { map })
    }
}

#[derive(Debug)]
pub struct NamedIndents {
    pub name: syn::Ident,
    pub map: HashMap<String, syn::Ident>,
}
impl Parse for NamedIndents {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<syn::Ident>()?;
        let map = input.parse::<UnnamedIndents>()?; 

        Ok(Self {
            name,
            map: map.map,
        })
    }
}
// [-] [Indents]
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━


// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// [+] [List]
#[derive(Debug)]
pub struct UnnamedList<T> {
    pub list: Vec<T>,
}

impl<T: Parse> Parse for UnnamedList<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = syn::parenthesized!(content in input);
        let list = CommaSeparated::<T>::parse_terminated(&content)?;
        let list = list.into_iter().collect();

        Ok(Self { list })
    }
}

#[derive(Debug)]
pub struct NamedList<T> {
    pub name: syn::Ident,
    pub list: Vec<T>,
}

impl<T: Parse> Parse for NamedList<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<syn::Ident>()?;
        let list = input.parse::<NamedList<T>>()?; 

        Ok(Self {
            name,
            list: list.list,
        })
    }
}
// [-] [List]
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
