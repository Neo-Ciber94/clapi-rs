
/// Converts the expression into a `TokenStream`
macro_rules! quote_expr {
    ($value:expr) => {{
        let val = &$value;
        quote::quote!(#val)
    }};
}

/// Converts the `Option<T>` into a `TokenStream`
macro_rules! quote_option {
    ($value:expr) => {
        match $value {
            Some(n) => quote::quote! { Some(#n) },
            None => quote::quote! { None }
        }
    };
}