// Copied from clapi_macros::ext
use syn::{GenericArgument, PathArguments, Type};

macro_rules! is_primitive_type {
    ($name:ident, $t:ty) => {
        fn $name(&self) -> bool {
            self.is_type(stringify!($t))
                || self.is_type(concat!("std::primitive::", stringify!($t)))
                || self.is_type(concat!("core::primitive::", stringify!($t)))
        }
    };
}

/// Provides methods for check the type.
///
/// This methods are `string` base comparison, so may fail if the type is an type alias.
pub trait TypeExtensions {
    fn as_type(&self) -> &Type;
    fn path(&self) -> Option<String>;

    fn is_type(&self, ty: &str) -> bool {
        if let Some(path) = self.path() {
            path.as_str() == ty
        } else {
            false
        }
    }

    fn is_string(&self) -> bool {
        self.is_type("String")
            || self.is_type("std::string::String")
            || self.is_type("alloc::string::String")
    }

    fn is_option(&self) -> bool {
        if let Some(path) = self.path() {
            path == "Option" || path == "std::option::Option" || path == "core::option::Option"
        } else {
            false
        }
    }

    fn is_result(&self) -> bool {
        if let Some(path) = self.path() {
            path == "Result" || path == "std::result::Result" || path == "core::result::Result"
        } else {
            false
        }
    }

    fn is_vec(&self) -> bool {
        if let Some(path) = self.path() {
            path == "Vec" || path == "std::vec::Vec" || path == "alloc::vec::Vec"
        } else {
            false
        }
    }

    fn is_array(&self) -> bool {
        matches!(self.as_type(), Type::Array(_))
    }

    fn is_slice(&self) -> bool {
        match self.as_type() {
            Type::Slice(_) => true,
            Type::Reference(type_ref) => type_ref.elem.is_slice(),
            _ => false,
        }
    }

    fn generic_arguments(&self) -> Vec<GenericArgument> {
        if let Type::Path(type_path) = &self.as_type() {
            let last_segment = type_path.path.segments.last().unwrap();
            if let PathArguments::AngleBracketed(generics) = &last_segment.arguments {
                return generics
                    .args
                    .iter()
                    .cloned()
                    .collect::<Vec<GenericArgument>>();
            }
        }

        Vec::new()
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
    is_primitive_type!(is_i8, i8);
    is_primitive_type!(is_i16, i16);
    is_primitive_type!(is_i32, i32);
    is_primitive_type!(is_i64, i64);
    is_primitive_type!(is_i128, i128);
    is_primitive_type!(is_isize, isize);
    is_primitive_type!(is_f32, f32);
    is_primitive_type!(is_f64, f64);
}

impl TypeExtensions for Type {
    fn as_type(&self) -> &Type {
        &self
    }

    fn path(&self) -> Option<String> {
        if let Type::Path(type_path) = self {
            Some(
                type_path
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
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
    fn path_test() {
        assert_eq!(to_type(quote! { u8 }).path().unwrap().as_str(), "u8");
        assert_eq!(to_type(quote! { str }).path().unwrap().as_str(), "str");
        assert_eq!(to_type(quote! { bool }).path().unwrap().as_str(), "bool");
        assert_eq!(
            to_type(quote! { Person }).path().unwrap().as_str(),
            "Person"
        );
        assert_eq!(
            to_type(quote! { std::option::Option<String> })
                .path()
                .unwrap()
                .as_str(),
            "std::option::Option"
        );
        assert_eq!(
            to_type(quote! { lib::Runner }).path().unwrap().as_str(),
            "lib::Runner"
        );
    }

    #[test]
    fn is_primitive_test() {
        println!("{:?}", to_type(quote! { i8 }).path());
        assert!(to_type(quote! { str }).is_primitive());
        assert!(to_type(quote! { char }).is_primitive());
        assert!(to_type(quote! { bool }).is_primitive());
        assert!(to_type(quote! { i8 }).is_primitive());
        assert!(to_type(quote! { i16 }).is_primitive());
        assert!(to_type(quote! { i32 }).is_primitive());
        assert!(to_type(quote! { i64 }).is_primitive());
        assert!(to_type(quote! { i128 }).is_primitive());
        assert!(to_type(quote! { isize }).is_primitive());
        assert!(to_type(quote! { u8 }).is_primitive());
        assert!(to_type(quote! { u16 }).is_primitive());
        assert!(to_type(quote! { u32 }).is_primitive());
        assert!(to_type(quote! { u64 }).is_primitive());
        assert!(to_type(quote! { u128 }).is_primitive());
        assert!(to_type(quote! { usize }).is_primitive());

        assert!(!to_type(quote! { String }).is_primitive());
    }

    #[test]
    fn is_unsigned_integer_test() {
        assert!(to_type(quote! { i8 }).is_number());
        assert!(to_type(quote! { u8 }).is_integer());
        assert!(to_type(quote! { u8 }).is_unsigned_integer());

        assert!(to_type(quote! { u8 }).is_u8());
        assert!(to_type(quote! { u16 }).is_u16());
        assert!(to_type(quote! { u32 }).is_u32());
        assert!(to_type(quote! { u64 }).is_u64());
        assert!(to_type(quote! { u128 }).is_u128());
        assert!(to_type(quote! { usize }).is_usize());
    }

    #[test]
    fn is_signed_integer_test() {
        assert!(to_type(quote! { i8 }).is_number());
        assert!(to_type(quote! { i8 }).is_integer());
        assert!(to_type(quote! { i8 }).is_signed_integer());

        assert!(to_type(quote! { i8 }).is_i8());
        assert!(to_type(quote! { i16 }).is_i16());
        assert!(to_type(quote! { i32 }).is_i32());
        assert!(to_type(quote! { i64 }).is_i64());
        assert!(to_type(quote! { i128 }).is_i128());
        assert!(to_type(quote! { isize }).is_isize());
    }

    #[test]
    fn is_float_test() {
        assert!(to_type(quote! { f32 }).is_f32());
        assert!(to_type(quote! { f64 }).is_f64());
        assert!(to_type(quote! { f32 }).is_float());
        assert!(to_type(quote! { f64 }).is_float());
        assert!(to_type(quote! { f32 }).is_number());
        assert!(to_type(quote! { f64 }).is_number());
    }

    #[test]
    fn is_option_test() {
        assert!(to_type(quote! { Option<String> }).is_option());
        assert!(to_type(quote! { std::option::Option<String> }).is_option());
        assert!(to_type(quote! { core::option::Option<String> }).is_option());
    }

    #[test]
    fn is_result_test() {
        assert!(to_type(quote! { Result<String, u32> }).is_result());
        assert!(to_type(quote! { std::result::Result<String, u32> }).is_result());
        assert!(to_type(quote! { core::result::Result<String, u32> }).is_result());
    }

    #[test]
    fn is_vec_test() {
        assert!(to_type(quote! { Vec<String> }).is_vec());
        assert!(to_type(quote! { std::vec::Vec<u32> }).is_vec());
    }

    #[test]
    fn is_slice_test() {
        assert!(to_type(quote! { [u32] }).is_slice());
        assert!(to_type(quote! { &[u32] }).is_slice());
        assert!(to_type(quote! { &mut [u32] }).is_slice());
    }
}
