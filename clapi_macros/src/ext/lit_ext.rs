use syn::Lit;

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

impl LitExtensions for Lit {
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