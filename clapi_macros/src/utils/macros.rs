
/// Quote the result of an expression
macro_rules! quote_expr {
    ($value:expr) => {{
        let val = &$value;
        quote::quote!(#val)
    }};
}