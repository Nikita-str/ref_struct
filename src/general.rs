use std::{collections::HashMap, marker::PhantomData};
use syn::parse::{Parse, ParseStream};

type CommaSeparated<T> = syn::punctuated::Punctuated::<T, syn::Token![,]>;


// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
pub trait RepeatAllow {
    const REPEAT_ALLOWED: bool = false;
    const REPEAT_MSG: &'static str = "ident";
}

pub struct RepeatAllowStd;
impl RepeatAllow for RepeatAllowStd {}
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━


// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// [+] [Indents]
#[derive(Debug)]
pub struct UnnamedIdents<R: RepeatAllow = RepeatAllowStd> {
    pub map: HashMap<String, syn::Ident>,
    phantom: PhantomData<R>,
}
impl<R: RepeatAllow> Parse for UnnamedIdents<R> {
    fn parse(input: ParseStream) -> syn::Result<Self> {

        let content;
        let _ = syn::parenthesized!(content in input);
        let list = CommaSeparated::<syn::Ident>::parse_terminated(&content)?;

        let mut map = HashMap::new();
        for ident in list {
            let s = ident.to_string();
            let is_repeat = map.insert(s, ident.clone());
            if !R::REPEAT_ALLOWED && is_repeat.is_some() {
                let what_repeated = R::REPEAT_MSG;
                let ident = is_repeat.unwrap().to_string();
                panic!("repeated {what_repeated}: `{ident}`")
            } 
        }

        Ok(Self {
            map,
            phantom: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct NamedIdents<R: RepeatAllow = RepeatAllowStd> {
    pub name: syn::Ident,
    pub map: HashMap<String, syn::Ident>,
    phantom: PhantomData<R>,
}
impl<R: RepeatAllow> Parse for NamedIdents<R> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<syn::Ident>()?;
        let map = input.parse::<UnnamedIdents<R>>()?; 

        Ok(Self {
            name,
            map: map.map,
            phantom: PhantomData,
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
