use std::collections::HashMap;
use syn::parse::ParseStream;
use quote::quote;
use proc_macro2::TokenStream as TokenStream2;
use crate::general::{UnnamedList, UnnamedIndents};

const USE_COW: &str = "use_cow";
const PUB_CMD: &str = "public";
const DERIVE_CMD: &str = "derive";
const IGNORE_CMD: &str = "ignore";
const CLONE_CMD: &str = "clone";
const NAME_CMD: &str = "name";
const IGNORE_STRUCT_CMD: &str = "ignore_struct";

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// [+]   [CMD LOOP]-RELATED MACRO DEFINES
macro_rules! cmd_done_x {
    ([$input: ident]) => {{
        if $input.is_empty() { break }
        $input.parse::<syn::Token![,]>()?;
        continue
    }};
}
macro_rules! twiced_err_x {
    ([$input: ident] $cmd: expr) => {{
        let span = $input.span();
        let message = format!("twiced `{}(..)` cmd", $cmd);
        return Err(syn::Error::new(span, message))      
    }};
    ([$input: ident] ? $x: ident => $cmd: expr) => {{
        if $x.is_some() {
            twiced_err_x!([$input] $cmd)
        }
    }};
}
// [-]   [CMD LOOP]-RELATED MACRO DEFINES
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━


// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// [+]   SOME PARSE FNS
fn parse_derive(input: &ParseStream, derive: &mut Vec<TokenStream2>) -> syn::Result<()> {
    let path_list: UnnamedList<syn::Path> = input.parse()?;
    let mut add_derive = path_list.list.into_iter()
        .map(|path|quote!{ #path }).collect::<Vec<_>>();
    derive.append(&mut add_derive);
    Ok(())
}

fn parse_name(input: &ParseStream, unnamed_map: UnnamedIndents, name: &mut Option<syn::Ident>) -> syn::Result<()> {
    twiced_err_x!([input] ? name => NAME_CMD);
    if unnamed_map.map.len() != 1 {
        return Err(syn::Error::new(input.span(), "expected `name(struct_name)`"))
    }
    *name = Some(unnamed_map.map.into_iter().next().unwrap().1);
    Ok(())
}
fn err_unkn_cmd(input: &ParseStream, all_cmd: &[&str]) -> syn::Result<()> {
    let span = input.span();
    let message = format!("expected `cmd(..)` where cmd is one of {:?}", all_cmd);
    return Err(syn::Error::new(span, message))
}
// [-]   SOME PARSE FNS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━


// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// [+] [FULL ARGS]
#[derive(Debug)]
pub struct Args {
    pub name: Option<syn::Ident>,
    pub clone: HashMap<String, syn::Ident>,
    pub ignore: HashMap<String, syn::Ident>,
    pub derive: Vec<TokenStream2>,
    pub is_pub: bool,
    pub use_cow: bool,
    pub ignore_struct_args: Option<IgnoreArgs>,
}
impl syn::parse::Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        const ALL_CMD: &[&str] = &[NAME_CMD, CLONE_CMD, IGNORE_CMD,  DERIVE_CMD, PUB_CMD, USE_COW, IGNORE_STRUCT_CMD];

        let mut use_cow = false;
        let mut is_pub = false;
        let mut name = None;
        let mut ignore = None;
        let mut clone = None;
        let mut derive = Vec::new();
        let mut ignore_struct_args = None;

        loop {
            // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
            // [+]   LOOP RELATED MACRO REDEFINES
            macro_rules! cmd_done { ($($tt: tt)*) => { cmd_done_x!([input] $($tt)*) }; }
            macro_rules! twiced_err { ($($tt: tt)*) => { twiced_err_x!([input] $($tt)*) }; }
            // [-]   LOOP RELATED MACRO REDEFINES
            // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

            let cmd = input.parse::<syn::Ident>()?.to_string();
            match cmd.as_str() {
                DERIVE_CMD => { parse_derive(&input, &mut derive)?; }
                PUB_CMD => { is_pub = true; }
                USE_COW => { use_cow = true; }
                
                IGNORE_CMD | CLONE_CMD | NAME_CMD => {
                    let unnamed_map: UnnamedIndents = input.parse()?;
                    match cmd.as_str() {
                        IGNORE_CMD => {
                            twiced_err!(? ignore => IGNORE_CMD);
                            ignore = Some(unnamed_map.map);
                        }
                        CLONE_CMD => {
                            twiced_err!(? clone => CLONE_CMD);                    
                            clone = Some(unnamed_map.map);
                        }
                        NAME_CMD => { parse_name(&input, unnamed_map, &mut name)?; }

                        _ => unreachable!()
                    }
                }
                
                IGNORE_STRUCT_CMD => {
                    twiced_err!(? ignore_struct_args => IGNORE_STRUCT_CMD);

                    let content;
                    let _ = syn::parenthesized!(content in input);
                    ignore_struct_args = Some(content.parse::<IgnoreArgs>()?)
                }

                _ => { err_unkn_cmd(&input, ALL_CMD)?; }
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
            ignore_struct_args,
        })
    }
}
// [-] [FULL ARGS]
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━


// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// [+] [IGNORE STRUCT ARGS]
#[derive(Debug)]
pub struct IgnoreArgs {
    pub name: Option<syn::Ident>,
    pub derive: Vec<TokenStream2>,
}
impl syn::parse::Parse for IgnoreArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        const ALL_CMD: &[&str] = &[NAME_CMD, DERIVE_CMD];
        
        let mut name = None;
        let mut derive = Vec::new();

        loop {
            // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
            // [+]   LOOP RELATED MACRO REDEFINES
            macro_rules! cmd_done { ($($tt: tt)*) => { cmd_done_x!([input] $($tt)*) }; }
            // macro_rules! twiced_err { ($($tt: tt)*) => { twiced_err_x!([input] $($tt)*) }; }
            // [-]   LOOP RELATED MACRO REDEFINES
            // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

            let cmd = input.parse::<syn::Ident>()?.to_string();
            match cmd.as_str() {
                DERIVE_CMD => { parse_derive(&input, &mut derive)?; }
                NAME_CMD => {
                    let unnamed_map = input.parse()?;
                    parse_name(&input, unnamed_map, &mut name)?;
                }

                _ => { err_unkn_cmd(&input, ALL_CMD)?; }
            }

            cmd_done!()
        }

        Ok(IgnoreArgs {
            name,
            derive,
        })
    }
}
// [-] [IGNORE STRUCT ARGS]
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━