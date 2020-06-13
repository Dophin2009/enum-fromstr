extern crate darling;
extern crate proc_macro;
extern crate quote;
extern crate syn;

use darling::{ast, util, FromDeriveInput, FromVariant};
use proc_macro::TokenStream;
use quote::{format_ident, quote};

#[derive(Debug, FromDeriveInput)]
struct DerivedEnum {
    pub ident: syn::Ident,
    pub data: ast::Data<DerivedValue, util::Ignored>,
}

#[derive(Debug, FromVariant)]
struct DerivedValue {
    pub ident: syn::Ident,
}

#[proc_macro_derive(EnumFromStr)]
pub fn derive(input: TokenStream) -> TokenStream {
    let parsed = syn::parse(input).unwrap();
    let reciever = DerivedEnum::from_derive_input(&parsed).unwrap();

    let enum_name: &syn::Ident = &reciever.ident;
    let error_name = format_ident!("{}Error", enum_name.to_string());

    if let ast::Data::Enum(ref values) = reciever.data {
        let valids = values
            .iter()
            .map(|v: &DerivedValue| &v.ident)
            .collect::<std::vec::Vec<_>>();
        let valids_str = valids
            .iter()
            .map(|id| id.to_string())
            .collect::<std::vec::Vec<_>>()
            .join(", ");

        let valids_branches = valids.iter().map(|v| {
            let str_form = format!("{}", v).to_owned().to_lowercase();
            quote! {
                #str_form => std::result::Result::Ok(#enum_name::#v)
            }
        });

        (quote! {
            impl std::str::FromStr for #enum_name {
                type Err = #error_name;

                fn from_str(s: &str) -> std::result::Result<Self, #error_name> {
                    match s.to_lowercase().as_str() {
                        #(#valids_branches),*,
                        _ => std::result::Result::Err(#error_name(s.to_owned())),
                    }
                }
            }

            #[derive(Debug)]
            struct #error_name(String);

            impl std::error::Error for #error_name {}

            impl std::fmt::Display for #error_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{} is not a valid value for enum with values [{}]", self.0, #valids_str)
                }
            }
        })
        .into()
    } else {
        panic!();
    }
}
