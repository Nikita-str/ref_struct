use std::collections::HashSet;

use quote::quote;
use syn::DeriveInput;
use proc_macro2::TokenStream as TokenStream2;

pub struct StructInfo {
    pub name: syn::Ident,
    pub field_names_set: HashSet<String>,
    pub field_names: Vec<syn::Ident>,
    pub field_types: Vec<TokenStream2>,
}
impl StructInfo {
    pub fn from_derive_input(input: &DeriveInput) -> Self {
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

        let field_names_set = field_names.iter().map(|x|x.to_string()).collect();

        Self {
            name,
            field_names_set,
            field_names,
            field_types,
        }
    }
}