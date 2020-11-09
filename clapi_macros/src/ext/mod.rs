
pub trait IteratorExt: Iterator {
    fn single(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let mut ret: Option<Self::Item> = None;

        for item in self {
            if ret.is_some() {
                return None;
            } else {
                ret = Some(item);
            }
        }

        ret
    }
}

impl<I: Iterator> IteratorExt for I {}

pub trait LitExtensions {
    fn is_str_literal(&self) -> bool;
    fn is_byte_str_literal(&self) -> bool;
    fn is_byte_literal(&self) -> bool;
    fn is_bool_literal(&self) -> bool;
    fn is_char_literal(&self) -> bool;
    fn is_integer_literal(&self) -> bool;
    fn is_float_literal(&self) -> bool;
    fn is_verbatim(&self) -> bool;

    fn is_string(&self) -> bool {
        self.is_str_literal() || self.is_byte_str_literal()
    }

    fn is_number(&self) -> bool {
        self.is_integer_literal() && self.is_float_literal()
    }
}

impl LitExtensions for syn::Lit {
    fn is_str_literal(&self) -> bool {
        matches!(self, syn::Lit::Str(_))
    }

    fn is_byte_str_literal(&self) -> bool {
        matches!(self, syn::Lit::ByteStr(_))
    }

    fn is_byte_literal(&self) -> bool {
        matches!(self, syn::Lit::Byte(_))
    }

    fn is_bool_literal(&self) -> bool {
        matches!(self, syn::Lit::Bool(_))
    }

    fn is_char_literal(&self) -> bool {
        matches!(self, syn::Lit::Char(_))
    }

    fn is_integer_literal(&self) -> bool {
        matches!(self, syn::Lit::Int(_))
    }

    fn is_float_literal(&self) -> bool {
        matches!(self, syn::Lit::Float(_))
    }

    fn is_verbatim(&self) -> bool {
        matches!(self, syn::Lit::Verbatim(_))
    }
}

macro_rules! is_primitive_type {
    ($name:ident, $t:ty) => {
        fn $name(&self) -> bool {
            self.is_type(stringify!($t))
                || self.is_type(concat!("std::primitive::", stringify!($t)))
                || self.is_type(concat!("core::primitive::", stringify!($t)))
        }
    };
}

pub trait TypeExtensions {
    fn path(&self) -> Option<String>;

    fn is_type(&self, ty: &str) -> bool {
        if let Some(path) = self.path() {
            path.as_str() == ty
        } else {
            false
        }
    }

    fn is_string(&self) -> bool {
        self.is_type("String") || self.is_type("std::string::String")
    }

    fn is_option(&self) -> bool {
        if let Some(path) = self.path() {
            path == "Option" || path == "std::option::Option" || path == "core::option::Option"
        } else {
            false
        }
    }

    fn is_integer(&self) -> bool {
        self.is_unsigned_integer() || self.is_signed_integer()
    }

    fn is_unsigned_integer(&self) -> bool {
        self.is_u8()
        || self.is_u16()
        || self.is_u32()
        || self.is_u64()
        || self.is_u128()
        || self.is_usize()
    }

    fn is_signed_integer(&self) -> bool {
        self.is_i8()
            || self.is_i16()
            || self.is_i32()
            || self.is_i64()
            || self.is_i128()
            || self.is_isize()
    }

    fn is_float(&self) -> bool {
        self.is_f32() || self.is_f64()
    }

    fn is_number(&self) -> bool {
        self.is_signed_integer() || self.is_unsigned_integer() || self.is_float()
    }

    fn is_primitive(&self) -> bool {
        self.is_str()
            || self.is_char()
            || self.is_bool()
            || self.is_u8()
            || self.is_u16()
            || self.is_u32()
            || self.is_u64()
            || self.is_u128()
            || self.is_usize()
            || self.is_i8()
            || self.is_i16()
            || self.is_i32()
            || self.is_i64()
            || self.is_i128()
            || self.is_isize()
            || self.is_f32()
            || self.is_f64()
    }

    is_primitive_type!(is_str, str);
    is_primitive_type!(is_char, char);
    is_primitive_type!(is_bool, bool);
    is_primitive_type!(is_u8, u8);
    is_primitive_type!(is_u16, u16);
    is_primitive_type!(is_u32, u32);
    is_primitive_type!(is_u64, u64);
    is_primitive_type!(is_u128, u128);
    is_primitive_type!(is_usize, usize);
    is_primitive_type!(is_i8, u8);
    is_primitive_type!(is_i16, u16);
    is_primitive_type!(is_i32, u32);
    is_primitive_type!(is_i64, u64);
    is_primitive_type!(is_i128, u128);
    is_primitive_type!(is_isize, isize);
    is_primitive_type!(is_f32, f32);
    is_primitive_type!(is_f64, f64);
}

impl TypeExtensions for syn::Type {
    fn path(&self) -> Option<String> {
        if let syn::Type::Path(type_path) = self {
            use syn::export::ToTokens;

            Some(
                type_path.to_token_stream()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join("::"),
            )
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use quote::*;
    use syn::Type;

    fn to_type(stream: TokenStream) -> Type {
        use syn::parse::Parse;
        syn::parse::Parser::parse2(Type::parse, stream).unwrap()
    }

    #[test]
    fn is_type_test() {
        let u8_type = to_type(quote! { u8 });
        assert!(u8_type.is_u8());
        assert!(u8_type.is_type("u8"))
    }

    #[test]
    fn is_option_test(){
        let u8_type = to_type(quote! { Option<String> });
        assert!(u8_type.is_type("Option"));
        assert!(u8_type.is_type("std::option::Option"));
        assert!(u8_type.is_type("core::option::Option"));
    }
}
