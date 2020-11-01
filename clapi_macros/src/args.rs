use crate::attr_data::{literal_to_string, AttributeData, Value};
use crate::parser::{parse_to_str_stream2, parse_to_stream2};
use crate::var::ArgType;
use proc_macro2::TokenStream;
use quote::*;
use syn::PatType;

/// Tokens for:
///
/// ```ignore
/// #[args(
///     name="args",
///     min=0,
///     max=100,
///     default=1,2,3
/// )]
/// ```
#[derive(Debug)]
pub struct ArgAttribute {
    min: Option<usize>,
    max: Option<usize>,
    pat_type: Option<PatType>,
    default_values: Vec<String>,
}

impl ArgAttribute {
    pub fn new(pat_type: Option<PatType>) -> Self {
        ArgAttribute {
            min: None,
            max: None,
            pat_type,
            default_values: vec![],
        }
    }

    pub fn from_attribute_data(attr: AttributeData, pat_type: Option<PatType>) -> ArgAttribute {
        new_arg_tokens_from_attr_data(attr, pat_type)
    }

    pub fn has_default_values(&self) -> bool {
        !self.default_values.is_empty()
    }

    pub fn set_min(&mut self, min: usize) {
        self.min = Some(min);
    }

    pub fn set_max(&mut self, max: usize) {
        self.max = Some(max);
    }

    pub fn set_default_values(&mut self, default_values: Vec<String>) {
        assert!(default_values.len() > 0);
        self.default_values = default_values;
    }

    pub fn expand(&self) -> TokenStream {
        let is_array = self
            .pat_type
            .as_ref()
            .map(ArgType::new)
            .map(|a| a.is_array())
            .unwrap_or(false);

        let min = if is_array {
            self.min.unwrap_or(0)
        } else {
            self.min.unwrap_or(1)
        };

        let max = if is_array {
            self.min.unwrap_or(usize::max_value())
        } else {
            self.min.unwrap_or(1)
        };

        assert!(min <= max, "invalid args `key` values");

        if !is_array {
            assert_eq!(
                min,
                1,
                "only `vec` and `slice` allow multiple values: `{}`",
                self.pat_type.to_token_stream().to_string()
            );
            assert_eq!(
                max,
                1,
                "only `vec` and `slice` allow multiple values: `{}`",
                self.pat_type.to_token_stream().to_string()
            );
        }

        if !self.default_values.is_empty() {
            assert!(
                (min..=max).contains(&self.default_values.len()),
                "invalid default values count, expected from `{}` to `{}` values",
                min,
                max
            );
        }

        let min = parse_to_stream2(min).unwrap();
        let max = parse_to_stream2(max).unwrap();

        let default_values = if self.default_values.is_empty() {
            quote! {}
        } else {
            let tokens = self
                .default_values
                .iter()
                .map(|s| parse_to_str_stream2(s).unwrap());
            quote! { .set_default_values(&[#(#tokens),*]) }
        };

        quote! {
            clapi::args::Arguments::new(clapi::arg_count::ArgCount::new(#min, #max))
            #default_values
        }
    }
}

impl ToTokens for ArgAttribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.expand().into_iter())
    }
}

fn new_arg_tokens_from_attr_data(attr: AttributeData, pat_type: Option<PatType>) -> ArgAttribute {
    let mut args = ArgAttribute::new(pat_type);

    for (key, value) in &attr {
        match key.as_str() {
            "name" => { /* Ignore */ }
            "min" => {
                let min = value
                    .clone()
                    .clone()
                    .parse_literal::<usize>()
                    .expect("option `min` is expected to be an integer literal");

                args.set_min(min);
            }
            "max" => {
                let max = value
                    .clone()
                    .clone()
                    .parse_literal::<usize>()
                    .expect("option `max` is expected to be an integer literal");

                args.set_max(max);
            }
            "default" => match value {
                Value::Literal(lit) => {
                    let s = literal_to_string(lit);
                    args.set_default_values(vec![s])
                }
                Value::Array(_) => {
                    let array = value.parse_array::<String>().unwrap();
                    args.set_default_values(array)
                }
                _ => panic!("option `default` expected to be literal or array"),
            },
            _ => panic!("invalid {} key `{}`", attr.path(), key),
        }
    }

    args
}
