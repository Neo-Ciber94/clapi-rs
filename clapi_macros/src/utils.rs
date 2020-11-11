use syn::PatType;
use syn::export::ToTokens;

/// Quote the result of an expression
macro_rules! quote_expr {
    ($value:expr) => {{
        let val = &$value;
        quote::quote!(#val)
    }};
}

pub fn pat_type_to_string(pat_type: &PatType) -> String {
    let arg_name = pat_type.pat.to_token_stream().to_string();
    let type_name = pat_type.ty.to_token_stream().into_iter()
        .map(|t| t.to_string())
        .collect::<Vec<String>>()
        .join("");

    format!("{} : {}", arg_name, type_name)
}