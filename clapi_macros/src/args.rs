use crate::utils::pat_type_to_string;
use crate::var::ArgType;
use crate::{LitExtensions, TypeExtensions, keys};
use macro_attribute::{literal_to_string, NameValueAttribute, Value};
use proc_macro2::TokenStream;
use quote::*;
use syn::{Lit, PatType};

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
    name: Option<String>,
    min: Option<usize>,
    max: Option<usize>,
    fn_arg: Option<NamedFnArg>,
    default_values: Vec<Lit>,
}

impl ArgData {
    pub fn new() -> Self {
        ArgData {
            min: None,
            max: None,
            fn_arg: None,
            name: None,
            default_values: vec![],
        }
    }

    pub fn from_pat_type(pat_type: &PatType) -> Self {
        let fn_arg = NamedFnArg::new(pat_type);
        let name = fn_arg.name.clone(); // use the same var name

        ArgData {
            min: None,
            max: None,
            fn_arg: Some(fn_arg),
            name: Some(name),
            default_values: vec![],
        }
    }

    pub fn from_attribute(attr: NameValueAttribute, pat_type: &PatType) -> ArgData {
        new_arg_tokens_from_attr_data(attr, pat_type)
    }

    pub fn has_default_values(&self) -> bool {
        !self.default_values.is_empty()
    }

    pub fn set_name(&mut self, name: Option<String>) {
        self.name = name;
    }

    pub fn set_min(&mut self, min: usize) {
        self.min = Some(min);
    }

    pub fn set_max(&mut self, max: usize) {
        self.max = Some(max);
    }

    pub fn set_default_values(&mut self, default_values: Vec<Lit>) {
        if let Some(arg) = self.fn_arg.as_ref() {
            assert_same_type_default_values(&arg.name, default_values.as_slice());
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

        let default_values = if self.default_values.is_empty() {
            quote! {}
        } else {
            let tokens = self.default_values.iter().map(|s| quote! { #s });
            quote! { .set_default_values(&[#(#tokens),*]) }
        };

        let name = self.name.as_ref()
            .map(|s| quote!{ .set_name(#s)} )
            .unwrap_or_else(|| quote!{ });

        quote! {
            clapi::args::Arguments::new(clapi::arg_count::ArgCount::new(#min, #max))
            #name
            #default_values
        }
    }

    fn arg_count(&self) -> (usize, usize) {
        let arg = if let Some(named_arg) = self.fn_arg.as_ref() {
          named_arg
        } else {
            let min = self.min.expect("`min` argument count is not defined");
            let max = self.max.expect("`max` argument count is not defined");
            assert!(min <= max, "invalid arguments range `min` cannot be greater than `max`");
            return (min, max);
        };

        let (min, max) = match (self.min, self.max) {
            (Some(min), Some(max)) => (min, max),
            (Some(min), None) => (min, usize::max_value()),
            (None, Some(max)) => (0, max),
            (None, None) => match arg.ty {
                ArgType::Raw(_) => (1, 1),
                ArgType::Option(_) => (0, 1),
                ArgType::Vec(_) | ArgType::Slice(_) | ArgType::MutSlice(_) => {
                    (0, usize::max_value())
                }
            },
        };

        assert!(min <= max, "invalid arguments range `min` cannot be greater than `max`");

        match arg.ty {
            ArgType::Raw(_) => {
                if min != 1 || max != 1 {
                    panic!("invalid number of arguments for `{}` expected 1", pat_type_to_string(&arg.pat_type));
                }
                (min, max)
            }
            ArgType::Option(_) => {
                if min != 0 || max != 1{
                    panic!("invalid number of arguments for `{}` expected 0 or 1", pat_type_to_string(&arg.pat_type));
                }
                (min, max)
            }
            ArgType::Vec(_) | ArgType::Slice(_) | ArgType::MutSlice(_) => (min, max),
        }
    }
}

impl ToTokens for ArgData {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.expand().into_iter())
    }
}

#[derive(Debug)]
struct NamedFnArg {
    name: String,
    pat_type: PatType,
    ty: ArgType,
}

impl NamedFnArg {
    pub fn new(pat_type: &PatType) -> Self {
        let name = if let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
            pat_ident.ident.to_string()
        } else {
            unreachable!()
        };

        NamedFnArg {
            name,
            pat_type: pat_type.clone(),
            ty: ArgType::new(pat_type),
        }
    }
}

fn new_arg_tokens_from_attr_data(attr: NameValueAttribute, pat_type: &PatType) -> ArgData {
    let mut args = ArgData::from_pat_type(pat_type);

    for (key, value) in &attr {
        match key.as_str() {
            keys::NAME => { /* Ignore */ }
            keys::MIN => {
                let min = value
                    .clone()
                    .parse_literal::<usize>()
                    .expect("option `min` is expected to be an integer literal");

                args.set_min(min);
            }
            keys::MAX => {
                let max = value
                    .clone()
                    .parse_literal::<usize>()
                    .expect("option `max` is expected to be an integer literal");

                args.set_max(max);
            }
            keys::DEFAULT => match value {
                Value::Literal(lit) => args.set_default_values(vec![lit.clone()]),
                Value::Array(array) => args.set_default_values(array.clone()),
            },
            _ => panic!("invalid {} key `{}`", attr.path(), key),
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

fn assert_arg_and_default_values_same_type(arg: &NamedFnArg, default_values: &[Lit]) {
    let ty = arg.ty.get_type();

    if cfg!(debug_assertions) {
        assert_same_type_default_values(&arg.name, default_values);
    }

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

    if ty.is_bool() {
        assert!(
            lit.is_bool_literal(),
            "expected bool default value for `{}` but was `{}`",
            arg.name,
            lit_str
        );
    } else if ty.is_char() {
        assert!(
            lit.is_char_literal(),
            "expected char default value for `{}` but was `{}`",
            arg.name,
            lit_str
        );
    } else if ty.is_string() {
        assert!(
            lit.is_string(),
            "expected string default value for `{}` but was `{}`",
            arg.name,
            lit_str
        );
    } else if ty.is_integer() {
        assert!(
            lit.is_integer_literal(),
            "expected integer default value for `{}` but was `{}`",
            arg.name,
            lit_str
        )
    } else if ty.is_float() {
        assert!(
            lit.is_integer_literal(),
            "expected float default value for `{}` but was `{}`",
            arg.name,
            lit_str
        )
    }
}