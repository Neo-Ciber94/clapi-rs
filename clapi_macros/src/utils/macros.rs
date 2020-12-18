
/// Quote the result of an expression
macro_rules! quote_expr {
    ($value:expr) => {{
        let val = &$value;
        quote::quote!(#val)
    }};
}

/// Matches the expression and returns `Some(ret)`.
macro_rules! matches_map {
    ($expression:expr, $pattern:pat => $ret:expr) => {
        match $expression {
            $pattern => Some($ret),
            _ => None,
        }
    };
}