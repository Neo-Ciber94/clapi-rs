#![allow(clippy::len_zero, clippy::redundant_closure)]
use crate::command::{is_option_bool_flag, FnArgData};
use crate::macro_attribute::{lit_to_string, Value, MacroAttribute};
use crate::utils::pat_type_to_string;
use crate::var::ArgumentType;
use crate::{attr, LitExtensions, TypeExtensions};
use proc_macro2::TokenStream;
use quote::*;
use syn::Lit;

/// Tokens for an `arg` attribute.
///
/// ```text
/// #[command]
/// #[arg(numbers, description="Numbers to sum", min=0, max=100, default=0)]
/// fn sum(numbers: Vec<i64>){
///     println!("Total: {}", numbers.iter().sum::<i64>());
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ArgAttrData {
    name: String,
    min: Option<usize>,
    max: Option<usize>,
    description: Option<String>,
    fn_arg: (FnArgData, ArgumentType),
    default_values: Vec<Lit>,
    attribute: Option<MacroAttribute>,
}

impl ArgAttrData {
    pub fn from_arg_data(arg_data: FnArgData) -> Self {
        // Deconstruct the `FnArgData`
        let FnArgData {
            arg_name,
            pat_type,
            attribute,
            name_value,
            ..
        } = arg_data.clone();

        let mut arg = ArgAttrData {
            name: arg_name,
            min: None,
            max: None,
            description: None,
            fn_arg: (arg_data, ArgumentType::new(&pat_type)),
            default_values: vec![],
            attribute
        };

        if let Some(attribute) = name_value {
            for (key, value) in attribute {
                match key.as_str() {
                    attr::ARG => {
                        let name = value
                            .to_string_literal()
                            .expect("arg `arg` must be a string literal");

                        arg.set_name(name);
                    }
                    attr::MIN => {
                        let min = value
                            .to_integer_literal::<usize>()
                            .expect("arg `min` must be an integer literal");

                        arg.set_min(min);
                    }
                    attr::MAX => {
                        let max = value
                            .to_integer_literal::<usize>()
                            .expect("arg `max` must be an integer literal");

                        arg.set_max(max);
                    }
                    attr::DESCRIPTION => {
                        let description = value
                            .to_string_literal()
                            .expect("arg `description` is expected to be a string literal");

                        arg.set_description(description);
                    }
                    attr::DEFAULT => match value {
                        Value::Literal(lit) => arg.set_default_values(vec![lit]),
                        Value::Array(array) => arg.set_default_values(array),
                    },
                    _ => panic!("invalid `arg` key `{}`", key),
                }
            }
        }

        arg
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn has_default_values(&self) -> bool {
        !self.default_values.is_empty()
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_min(&mut self, min: usize) {
        self.min = Some(min);
    }

    pub fn set_max(&mut self, max: usize) {
        self.max = Some(max);
    }

    pub fn set_description(&mut self, description: String) {
        self.description = Some(description)
    }

    pub fn set_default_values(&mut self, default_values: Vec<Lit>) {
        assert!(default_values.len() > 0, "default values is empty");
        if let Err(diff) = check_same_type(&default_values[0], default_values.as_slice()) {
            panic!(
                "invalid default value for arg `{}`, expected `{}` but was `{}`.\
                Default values must be of the same type",
                self.name,
                lit_variant_to_string(&default_values[0]),
                lit_variant_to_string(diff)
            )
        }
        self.default_values = default_values;
    }

    pub fn expand(&self) -> TokenStream {
        if self.has_default_values() {
            assert_arg_and_default_values_same_type(&self.fn_arg, &self.default_values);
        }

        let (min, max) = self.arg_count();

        // Assertions
        self.assert_min_max(min, max);
        self.assert_default_values_range(min, max);

        // Argument count
        let min = quote_option!(min);
        let max = quote_option!(max);

        let arg_count = quote! {
            .arg_count(clapi::ArgCount::new_checked(#min, #max).expect("min < max"))
        };

        // Argument default values
        let default_values = if self.default_values.is_empty() {
            quote! {}
        } else {
            let tokens = self.default_values.iter().map(|s| quote! { #s });
            quote! { .defaults(&[#(#tokens),*]) }
        };

        // Argument description
        let description = self
            .description
            .as_ref()
            .map(|s| quote! { .description(#s)})
            .unwrap_or_else(|| quote! {});

        // Argument name
        let name = quote_expr!(self.name);

        quote! {
            clapi::Argument::new(#name)
            #arg_count
            #description
            #default_values
        }
    }

    fn arg_count(&self) -> (Option<usize>, Option<usize>) {
        let (arg, arg_type) = &self.fn_arg;

        // Get the `min` and `max` number of values for this argument.
        let (min, max) = match (self.min, self.max) {
            (min, max) if min.is_some() | max.is_none() => (min, max),
            _ => match arg_type {
                ArgumentType::Type(_) => (Some(1), Some(1)),
                ArgumentType::Option(_) => (Some(0), Some(1)),
                ArgumentType::Vec(_) | ArgumentType::Slice(_) => (Some(0), None),
                ArgumentType::Array(array) => (Some(array.len), Some(array.len))
            },
        };

        assert_valid_arg_count(&arg, arg_type, min, max);
        (min, max)
    }

    fn assert_min_max(&self, min: Option<usize>, max: Option<usize>) {
        if let (Some(min), Some(max)) = (min, max) {
            if min > max {
                panic!("invalid argument count `min` cannot be greater than `max`")
            }
        }
    }

    fn assert_default_values_range(&self, min: Option<usize>, max: Option<usize>) {
        if self.default_values.is_empty() {
            return;
        }

        let len = self.default_values.len();

        match (min, max) {
            (Some(min), Some(max)) if !(min..=max).contains(&len) => {
                panic!("invalid default values count, expected from {} to {} values but was {}", min, max, len);
            },
            (Some(min), None) if !(min..).contains(&len) => {
                panic!("invalid default values count, expected {} or more values but was {}", min, len);
            },
            (None, Some(max)) if !(..=max).contains(&len) => {
                panic!("invalid default values count, expected {} or less values but was {}", max, len);
            }
            _ => {}
        }
    }
}

impl ToTokens for ArgAttrData {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.expand().into_iter())
    }
}

impl Eq for ArgAttrData {}

impl PartialEq for ArgAttrData {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

fn lit_variant_to_string(lit: &Lit) -> &'static str {
    match lit {
        Lit::Str(_) => "string",
        Lit::ByteStr(_) => "string",
        Lit::Byte(_) => "byte",
        Lit::Char(_) => "char",
        Lit::Int(_) => "integer",
        Lit::Float(_) => "float",
        Lit::Bool(_) => "bool",
        Lit::Verbatim(_) => "verbatim",
    }
}

fn check_same_type<'a>(left: &'a Lit, right: &'a [Lit]) -> Result<(), &'a Lit> {
    if right.len() <= 1 {
        Ok(())
    } else {
        for value in right {
            if std::mem::discriminant(left) != std::mem::discriminant(value) {
                return Err(value);
            }
        }

        Ok(())
    }
}

fn assert_arg_and_default_values_same_type(
    (arg, ty): &(FnArgData, ArgumentType),
    default_values: &[Lit],
) {
    let arg_type = ty.get_type();
    let lit = &default_values[0];

    let lit_str = if default_values.len() > 1 {
        let s = default_values
            .iter()
            .map(lit_to_string)
            .collect::<Vec<String>>()
            .join(", ");

        format!("[{}]", s)
    } else {
        lit_to_string(&default_values[0])
    };

    if arg_type.is_bool() {
        assert!(
            lit.is_bool_literal(),
            "expected bool default value for `{}` but was `{}`",
            arg.arg_name,
            lit_str
        );
    } else if arg_type.is_char() {
        assert!(
            lit.is_char_literal(),
            "expected char default value for `{}` but was `{}`",
            arg.arg_name,
            lit_str
        );
    } else if arg_type.is_string() {
        assert!(
            lit.is_string(),
            "expected string default value for `{}` but was `{}`",
            arg.arg_name,
            lit_str
        );
    } else if arg_type.is_integer() {
        assert!(
            lit.is_integer_literal(),
            "expected integer default value for `{}` but was `{}`",
            arg.arg_name,
            lit_str
        )
    } else if arg_type.is_float() {
        assert!(
            lit.is_integer_literal(),
            "expected float default value for `{}` but was `{}`",
            arg.arg_name,
            lit_str
        )
    }
}

fn assert_valid_arg_count(arg: &FnArgData, arg_type: &ArgumentType, min: Option<usize>, max: Option<usize>) {
    // We don't check if there is no `min` and `max`
    if min.is_none() && max.is_none() {
        return;
    }

    let min = min.unwrap_or(0);
    let max = max.unwrap_or(usize::max_value());

    match arg_type {
        ArgumentType::Type(_) => {
            // If the argument is an option bool flag, `min` and `max` will be 0 and 1.
            // Check `option.rs`
            if !is_option_bool_flag(arg) {
                if min != 1 || max != 1{
                    panic!(
                        "invalid number of arguments for `{}` expected 1",
                        pat_type_to_string(&arg.pat_type)
                    );
                }
            }
        }
        ArgumentType::Option(_) => {
            if min != 0 || max != 1{
                panic!(
                    "invalid number of arguments for `{}` expected from 0 to 1",
                    pat_type_to_string(&arg.pat_type),
                );
            }
        }
        ArgumentType::Vec(_) | ArgumentType::Slice(_) => { /* Nothing */ },
        ArgumentType::Array(array) => {
            if min != max {
                panic!("invalid number of arguments for `{}` expected {}",
                    pat_type_to_string(&arg.pat_type),
                    array.len
                );
            }
        }
    }
}