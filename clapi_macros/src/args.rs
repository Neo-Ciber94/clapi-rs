use crate::attr_data::AttributeData;
use crate::command::TypedFnArg;
use crate::{parse_to_str_stream2, parse_to_stream2};
use clapi::arg_count::ArgCount;
use clapi::utils::{Also, Then};
use proc_macro2::TokenStream;
use quote::*;
use syn::{Attribute, Signature, Type};

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

    pub fn from(fn_arg: TypedFnArg, attribute: Option<AttributeData>) -> Self {
        let mut args_tokens = ArgsTokens::new();
        args_tokens.set_arg_type(fn_arg.ty);

        if let Some(attr) = attribute {
            assert_eq!(attr.path(), "args", "expected attribute named `args`");

            for (key, value) in attr {
                match key.as_str() {
                    "min" => {
                        assert!(args_tokens.min.is_none(), "duplicated args key `min`");
                        let min = value
                            .parse_literal::<usize>()
                            .expect("expected number literal for `min`");

                        args_tokens.set_min(min);
                    }
                    "max" => {
                        assert!(args_tokens.max.is_none(), "duplicated args key `max`");
                        let max = value
                            .parse_literal::<usize>()
                            .expect("expected number literal for `max`");

                        args_tokens.set_max(max);
                    }
                    "default" => {
                        assert!(args_tokens.default_values.is_empty(), "duplicated args key `default`");
                        let default = value
                            .clone()
                            .into_array()
                            .expect("expect literal or array for `default`");

                        args_tokens.set_default_values(default);
                    }
                    _ => panic!(
                        "unknown args key `{}`. allowed keys: `min`, `max`, `default`",
                        key
                    ),
                }
            }
        }

        args_tokens
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
