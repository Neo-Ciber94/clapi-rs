use proc_macro2::TokenStream;
use quote::*;
use syn::Lit;
use macro_attribute::{literal_to_string, Value};
use crate::{attr, LitExtensions, TypeExtensions};
use crate::command::{FnArgData, is_option_bool_flag};
use crate::utils::pat_type_to_string;
use crate::var::ArgumentType;

/// Tokens for:
///
/// ```text
/// #[args(
///     name="args",
///     min=0,
///     max=100,
///     default=1,2,3
/// )]
/// ```
#[derive(Debug)]
pub struct ArgData {
    name: String,
    min: Option<usize>,
    max: Option<usize>,
    fn_arg: Option<(FnArgData, ArgumentType)>,
    default_values: Vec<Lit>,
}

impl ArgData {
    pub fn with_name(name: String) -> Self {
        ArgData {
            name,
            min: None,
            max: None,
            fn_arg: None,
            default_values: vec![],
        }
    }

    pub fn new(arg: &FnArgData) -> ArgData {
        new_arg_data(arg)
    }

    pub fn has_default_values(&self) -> bool {
        !self.default_values.is_empty()
    }

    pub fn set_name(&mut self, name: String){
        self.name = name;
    }

    pub fn set_min(&mut self, min: usize) {
        self.min = Some(min);
    }

    pub fn set_max(&mut self, max: usize) {
        self.max = Some(max);
    }

    pub fn set_default_values(&mut self, default_values: Vec<Lit>) {
        if let Some((arg, _)) = self.fn_arg.as_ref() {
            assert_same_type_default_values(&arg.arg_name, default_values.as_slice());
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
                min, max
            );
        }

        let min = quote!{ #min };
        let max = quote!{ #max };

        let arg_count = quote! {
            .arg_count(#min..=#max)
        };

        let default_values = if self.default_values.is_empty() {
            quote! {}
        } else {
            let tokens = self.default_values.iter().map(|s| quote! { #s });
            quote! { .defaults(&[#(#tokens),*]) }
        };

        let name = quote_expr!(self.name);

        quote! {
            clapi::Argument::new(#name)
            #arg_count
            #default_values
        }
    }

    fn arg_count(&self) -> (usize, usize) {
        fn max_arg_count_for_type(arg_type: &ArgumentType) -> usize {
            match arg_type {
                ArgumentType::Type(_) | ArgumentType::Option(_) => 1,
                ArgumentType::Vec(_) | ArgumentType::Slice(_) | ArgumentType::MutSlice(_) => {
                    usize::max_value()
                }
            }
        }

        let (arg, arg_type) = if let Some(named_arg) = self.fn_arg.as_ref() {
          named_arg
        } else {
            let min = self.min.expect("`min` argument count is not defined");
            let max = self.max.expect("`max` argument count is not defined");
            assert!(min <= max, "invalid arguments range `min` cannot be greater than `max`");
            return (min, max);
        };

        let (min, max) = match (self.min, self.max) {
            (Some(min), Some(max)) => (min, max),
            (Some(min), None) => (min, max_arg_count_for_type(arg_type)),
            (None, Some(max)) => (0, max),
            (None, None) => match arg_type {
                ArgumentType::Type(_) => (1, 1),
                ArgumentType::Option(_) => (0, 1),
                ArgumentType::Vec(_) | ArgumentType::Slice(_) | ArgumentType::MutSlice(_) => {
                    (0, usize::max_value())
                }
            },
        };

        assert!(min <= max, "invalid arguments range `min` cannot be greater than `max`");

        match arg_type {
            ArgumentType::Type(_) => {
                // bool flag don't need check because are handler internally in `command`
                if !is_option_bool_flag(arg) {
                    if min != 1 {
                        panic!("invalid `min` number of arguments for `{}` expected 1 but was {}",
                               pat_type_to_string(&arg.pat_type), min);
                    }

                    if max != 1 {
                        panic!("invalid `max` number of arguments for `{}` expected 1 but was {}",
                               pat_type_to_string(&arg.pat_type), max);
                    }
                }

                (min, max)
            }
            ArgumentType::Option(_) => {
                if min != 0 {
                    panic!("invalid `min` number of arguments for `{}` expected 0 but was {}",
                           pat_type_to_string(&arg.pat_type), min);
                }

                if max != 1{
                    panic!("invalid `max` number of arguments for `{}` expected 1 but was {}",
                           pat_type_to_string(&arg.pat_type), max);
                }

                (min, max)
            }
            ArgumentType::Vec(_) | ArgumentType::Slice(_) | ArgumentType::MutSlice(_) => (min, max),
        }
    }
}

impl ToTokens for ArgData {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.expand().into_iter())
    }
}

fn new_arg_data(arg: &FnArgData) -> ArgData {
    let fn_arg = (arg.clone(), ArgumentType::new(&arg.pat_type));

    let mut args = ArgData{
        name: arg.arg_name.clone(),
        fn_arg: Some(fn_arg),
        min: None,
        max: None,
        default_values: vec![]
    };

    if let Some(attribute) = &arg.attribute {
        for (key, value) in attribute {
            match key.as_str() {
                attr::ARG => {
                    let name = value
                        .clone()
                        .as_string_literal()
                        .expect("arg `arg` is expected to be a string literal");

                    args.set_name(name);
                }
                attr::MIN => {
                    let min = value
                        .clone()
                        .parse_literal::<usize>()
                        .expect("option `min` is expected to be an integer literal");

                    args.set_min(min);
                }
                attr::MAX => {
                    let max = value
                        .clone()
                        .parse_literal::<usize>()
                        .expect("option `max` is expected to be an integer literal");

                    args.set_max(max);
                }
                attr::DEFAULT => match value {
                    Value::Literal(lit) => args.set_default_values(vec![lit.clone()]),
                    Value::Array(array) => args.set_default_values(array.clone()),
                },
                _ => panic!("invalid {} key `{}`", attribute.path(), key),
            }
        }
    }

    args
}

fn assert_same_type_default_values(arg_name: &str, default_values: &[Lit]) {
    fn panic_different_types_with_name(arg_name: &str, left: &Lit, right: &Lit) {
        fn lit_type_to_string(lit: &Lit) -> &'static str {
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

        panic!(
            "invalid default value for argument `{}`, expected {} but was {}",
            arg_name,
            lit_type_to_string(left),
            lit_type_to_string(right)
        );
    }

    let panic_different_types =
        |left: &Lit, right: &Lit| panic_different_types_with_name(arg_name, left, right);

    assert!(default_values.len() > 0, "`default` is empty");

    let lit = &default_values[0];

    match lit {
        Lit::Str(_) => {
            for x in default_values.iter().skip(1) {
                if !matches!(x, Lit::Str(_)) {
                    panic_different_types(lit, x);
                }
            }
        }
        Lit::ByteStr(_) => {
            for x in default_values.iter().skip(1) {
                if !matches!(x, Lit::ByteStr(_)) {
                    panic_different_types(lit, x);
                }
            }
        }
        Lit::Byte(_) => {
            for x in default_values.iter().skip(1) {
                if !matches!(x, Lit::Byte(_)) {
                    panic_different_types(lit, x);
                }
            }
        }
        Lit::Char(_) => {
            for x in default_values.iter().skip(1) {
                if !matches!(x, Lit::Char(_)) {
                    panic_different_types(lit, x);
                }
            }
        }
        Lit::Int(_) => {
            for x in default_values.iter().skip(1) {
                if !matches!(x, Lit::Int(_)) {
                    panic_different_types(lit, x);
                }
            }
        }
        Lit::Float(_) => {
            for x in default_values.iter().skip(1) {
                if !matches!(x, Lit::Float(_)) {
                    panic_different_types(lit, x);
                }
            }
        }
        Lit::Bool(_) => {
            for x in default_values.iter().skip(1) {
                if !matches!(x, Lit::Bool(_)) {
                    panic_different_types(lit, x);
                }
            }
        }
        Lit::Verbatim(_) => {
            for x in default_values.iter().skip(1) {
                if !matches!(x, Lit::Verbatim(_)) {
                    panic_different_types(lit, x);
                }
            }
        }
    }
}

fn assert_arg_and_default_values_same_type((arg, ty): &(FnArgData, ArgumentType), default_values: &[Lit]) {
    let arg_type = ty.get_type();
    let lit = &default_values[0];

    let lit_str = if default_values.len() > 1 {
        let s = default_values
            .iter()
            .map(literal_to_string)
            .collect::<Vec<String>>()
            .join(", ");

        format!("[{}]", s)
    } else {
        literal_to_string(&default_values[0])
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