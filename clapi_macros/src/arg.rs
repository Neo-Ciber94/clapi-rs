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
    fn_arg: Option<(FnArgData, ArgumentType)>,
    default_values: Vec<Lit>,
    attribute: Option<MacroAttribute>,
}

impl ArgAttrData {
    pub fn new(name: String, attribute: Option<MacroAttribute>) -> Self {
        ArgAttrData {
            name,
            min: None,
            max: None,
            description: None,
            fn_arg: None,
            default_values: vec![],
            attribute,
        }
    }

    pub fn from_arg_data(arg_data: FnArgData) -> Self {
        let arg_type = ArgumentType::new(&arg_data.pat_type);
        let mut args = ArgAttrData::new(
            arg_data.arg_name.clone(),
            arg_data.attribute.clone()
        );

        if let Some(attribute) = &arg_data.name_value {
            for (key, value) in attribute {
                match key.as_str() {
                    attr::ARG => {
                        let name = value
                            .clone()
                            .to_string_literal()
                            .expect("arg `arg` must be a string literal");

                        args.set_name(name);
                    }
                    attr::MIN => {
                        let min = value
                            .clone()
                            .to_integer_literal::<usize>()
                            .expect("arg `min` must be an integer literal");

                        args.set_min(min);
                    }
                    attr::MAX => {
                        let max = value
                            .clone()
                            .to_integer_literal::<usize>()
                            .expect("arg `max` must be an integer literal");

                        args.set_max(max);
                    }
                    attr::DESCRIPTION => {
                        let description = value
                            .clone()
                            .to_string_literal()
                            .expect("arg `description` is expected to be a string literal");

                        args.set_description(description);
                    }
                    attr::DEFAULT => match value {
                        Value::Literal(lit) => args.set_default_values(vec![lit.clone()]),
                        Value::Array(array) => args.set_default_values(array.clone()),
                    },
                    _ => panic!("invalid `arg` key `{}`", key),
                }
            }
        }

        args.fn_arg = Some((arg_data, arg_type));
        args
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
            if let Some(arg) = self.fn_arg.as_ref() {
                assert_arg_and_default_values_same_type(arg, &self.default_values);
            }
        }

        let (min, max) = self.arg_count();

        if !self.default_values.is_empty() {
            assert!(
                (min..=max).contains(&self.default_values.len()),
                "invalid default values count, expected from `{}` to `{}` values",
                min,
                max
            );
        }

        let min = quote! { #min };
        let max = quote! { #max };

        let arg_count = quote! {
            .arg_count(#min..=#max)
        };

        let default_values = if self.default_values.is_empty() {
            quote! {}
        } else {
            let tokens = self.default_values.iter().map(|s| quote! { #s });
            quote! { .defaults(&[#(#tokens),*]) }
        };

        let description = self
            .description
            .as_ref()
            .map(|s| quote! { .description(#s)})
            .unwrap_or_else(|| quote! {});

        let name = quote_expr!(self.name);

        quote! {
            clapi::Argument::new(#name)
            #arg_count
            #description
            #default_values
        }
    }

    fn arg_count(&self) -> (usize, usize) {
        fn max_arg_count_for_type(arg_type: &ArgumentType) -> usize {
            match arg_type {
                ArgumentType::Type(_) | ArgumentType::Option(_) => 1,
                ArgumentType::Vec(_) | ArgumentType::Slice(_) | ArgumentType::Array(_) => {
                    usize::max_value()
                }
            }
        }

        let (arg, arg_type) = match self.fn_arg.as_ref() {
            Some((arg, arg_type)) => (arg, arg_type),
            None => {
                let min = self.min.expect("`min` argument count is not defined");
                let max = self.max.expect("`max` argument count is not defined");
                assert!(min <= max, "invalid arguments range `min` cannot be greater than `max`");
                // Return from the function
                return (min, max);
            }
        };

        // Get the `min` and `max` number of values for this argument.
        let (min, max) = match (self.min, self.max) {
            (Some(min), Some(max)) => (min, max),
            (Some(min), None) => (min, max_arg_count_for_type(arg_type)),
            (None, Some(max)) => (0, max),
            (None, None) => match arg_type {
                ArgumentType::Type(_) => (1, 1),
                ArgumentType::Option(_) => (0, 1),
                ArgumentType::Vec(_) | ArgumentType::Slice(_) => (0, usize::max_value()),
                ArgumentType::Array(array) => (array.len, array.len)
            },
        };

        assert!(min <= max, "invalid arguments range `min` cannot be greater than `max`");
        check_valid_arg_count(&arg, arg_type, min, max);
        (min, max)
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

fn check_valid_arg_count(arg: &&FnArgData, arg_type: &ArgumentType, min: usize, max: usize) {
    match arg_type {
        ArgumentType::Type(_) => {
            // If the argument is an option bool flag, `min` and `max` will be 0 and 1.
            // Check `option.rs`
            if !is_option_bool_flag(arg) {
                if min != 1 {
                    panic!(
                        "invalid `min` number of arguments for `{}` expected 1 but was {}",
                        pat_type_to_string(&arg.pat_type),
                        min
                    );
                }

                if max != 1 {
                    panic!(
                        "invalid `max` number of arguments for `{}` expected 1 but was {}",
                        pat_type_to_string(&arg.pat_type),
                        max
                    );
                }
            }
        }
        ArgumentType::Option(_) => {
            if min != 0 {
                panic!(
                    "invalid `min` number of arguments for `{}` expected 0 but was {}",
                    pat_type_to_string(&arg.pat_type),
                    min
                );
            }

            if max != 1 {
                panic!(
                    "invalid `max` number of arguments for `{}` expected 1 but was {}",
                    pat_type_to_string(&arg.pat_type),
                    max
                );
            }
        }
        ArgumentType::Vec(_) | ArgumentType::Slice(_) => { /* Nothing */ },
        ArgumentType::Array(_) => {
            if min != max {
                panic!(
                    "invalid `min` and `max` number of arguments for `{}`, expected the `min == max` but was: `{}` and `{}`",
                    pat_type_to_string(&arg.pat_type),
                    min,
                    max
                );
            }
        }
    }
}