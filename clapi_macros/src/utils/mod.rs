mod name_path;
pub use name_path::NamePath;

use syn::{ItemFn, Attribute};
use syn::parse_quote::ParseQuote;
use quote::ToTokens;

#[macro_use]
mod macros;

/// Returns a formatted `PatType` as `x : i32`.
pub fn pat_type_to_string(pat_type: &syn::PatType) -> String {
    let arg_name = pat_type.pat.to_token_stream().to_string();
    let type_name = pat_type
        .ty
        .to_token_stream()
        .into_iter()
        .map(|t| t.to_string())
        .collect::<Vec<String>>()
        .join("");

    format!("{} : {}", arg_name, type_name)
}

/// Returns the `Path` to string like: `std::vec::Vec`.
pub fn path_to_string(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<String>>()
        .join("::")
}

pub fn insert_allow_dead_code_attribute(item_fn: &mut ItemFn){
    let tokens = quote::quote! { #[allow(dead_code)] };
    let attribute = syn::parse::Parser::parse2(Attribute::parse, tokens).unwrap();
    item_fn.attrs.push(attribute);
}
