use crate::parse_to_stream2;
use proc_macro2::TokenStream;
use quote::*;
use syn::Type;

/// Tokens for:
///
/// ```ignore
/// #[args(
///     name="args",
///     min=0,
///     max=100,
///     default=[1,2,3]
/// )]
/// ```
#[derive(Default, Debug)]
pub struct ArgsTokens {
    min: Option<usize>,
    max: Option<usize>,
    ty: Option<Box<Type>>,
    default_values: Vec<String>,
}

impl ArgsTokens {
    pub fn new() -> Self {
        ArgsTokens::default()
    }

    pub fn has_default_values(&self) -> bool{
        !self.default_values.is_empty()
    }

    pub fn set_min(&mut self, min: usize){
        self.min = Some(min);
    }

    pub fn set_max(&mut self, max: usize){
        self.max = Some(max);
    }

    pub fn set_arg_type(&mut self, ty: Box<Type>) {
        self.ty = Some(ty);
    }

    pub fn set_default_values(&mut self, default_values: Vec<String>) {
        assert!(default_values.len() > 0);
        self.default_values = default_values;
    }

    pub fn expand(&self) -> TokenStream {
        let min = self.min.unwrap_or(0);
        let max = self.max.unwrap_or(usize::max_value());

        assert!(min <= max, "invalid args `key` values");

        assert!(
            (min..=max).contains(&self.default_values.len()),
            "invalid default values count, expected from `{}` to `{}` values",
            min,
            max
        );

        let min = parse_to_stream2(min);
        let max = parse_to_stream2(max);

        let default_values = if self.default_values.is_empty() {
            quote! {}
        } else {
            let tokens = self.default_values.iter().map(|s| parse_to_stream2(s));

            quote! {
                .set_default_values(&[#(#tokens),*])
            }
        };

        quote! {
            clapi::args::Arguments::new(#min..=#max)
            #default_values
        }
    }
}

impl ToTokens for ArgsTokens{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.expand().into_iter())
    }
}
