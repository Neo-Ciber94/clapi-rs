use crate::ext::TypeExtensions;
use proc_macro2::TokenStream;
use quote::*;
use syn::parse::{Parse, ParseStream};
use syn::{GenericArgument, Ident, Type};

/// Provides the tokens to construct a variable in `clapi::app!` macro handler.
pub struct DeclareVar {
    input: VarInput,
    source: VarSource,
    ty: VarType,
}

impl DeclareVar {
    pub fn new(input: VarInput, source: VarSource) -> Self {
        let ty = get_var_type(&input.ty);
        DeclareVar { input, source, ty }
    }

    pub fn expand(&self) -> TokenStream {
        match self.source {
            VarSource::Option => self.declare_option_var(),
            VarSource::Argument => self.declare_argument_var(),
        }
    }

    fn declare_option_var(&self) -> TokenStream {
        let source = &self.input.source;
        let option_name = self.input.name.to_string();

        match &self.ty {
            VarType::Type(ty) if ty.is_bool() => {
                quote! {
                    match #source.get(#option_name){
                        None => false,
                        Some(option) => {
                            let arg = option.get_arg().unwrap();
                            if !arg.get_values().is_empty() {
                                arg.convert::<bool>()?
                            } else {
                                true
                            }
                        }
                    }
                }
            }
            VarType::Type(ty) => {
                quote! {
                    #source.get(#option_name)
                        .unwrap()
                        .get_arg()
                        .unwrap()
                        .convert::<#ty>()?
                }
            }
            VarType::Vec(ty) => {
                quote! {
                    #source.get(#option_name)
                        .unwrap()
                        .get_args()
                        .get_raw_args_as_type::<#ty>()?
                }
            }
            VarType::Option(ty) => {
                quote! {
                    match #source.get(#option_name) {
                        Some(option) => Some(option.get_arg().unwrap().convert::<#ty>()?),
                        None => None,
                    }
                }
            }
            VarType::Slice(ty) | VarType::MutSlice(ty) => {
                if matches!(self.ty, VarType::Slice(_)) {
                    panic!(
                        "slices are no supported, used `Vec<{0}>` instead: &[{0}]",
                        ty.to_token_stream().to_string()
                    )
                } else {
                    panic!(
                        "slices are no supported, used `Vec<{0}>` instead: &mut [{0}]",
                        ty.to_token_stream().to_string()
                    )
                }
            }
        }
    }

    fn declare_argument_var(&self) -> TokenStream {
        let source = &self.input.source;
        let arg_name = self.input.name.to_string();

        match &self.ty {
            VarType::Type(ty) => {
                let type_name = ty.to_token_stream().to_string();
                let msg = format!(
                    "multiple arguments defined, expected `Vec<{0}>` but was `{0}`",
                    type_name
                );
                quote! {
                    {
                        if #source.len() == 1 {
                            #source.get_raw_args_as_type::<#ty>()?.pop().unwrap()
                        } else {
                            panic!(#msg)
                        }
                    }
                }
            }
            VarType::Vec(ty) => {
                quote! {
                    #source.get_raw_args_as_type::<#ty>()?
                }
            }
            VarType::Option(ty) => {
                quote! {
                    match #source.get(#arg_name) {
                        Some(arg) => Some(arg.convert::<#ty>()?),
                        None => None,
                    }
                }
            }
            VarType::Slice(ty) | VarType::MutSlice(ty) => {
                if matches!(self.ty, VarType::Slice(_)) {
                    panic!(
                        "slices are not supported, use `Vec<{0}>` instead of `&[{0}]`",
                        ty.to_token_stream().to_string()
                    )
                } else {
                    panic!(
                        "slices are not supported, use `Vec<{0}>` instead of `&mut [{0}]`",
                        ty.to_token_stream().to_string()
                    )
                }
            }
        }
    }
}

/// Represents the declaration of a variable in `clapi::app!` macro like:
/// `(source, mut? name : i64)`
#[derive(Debug)]
pub struct VarInput {
    source: Ident,
    comma: syn::Token![,],
    mutability: Option<syn::Token![mut]>,
    name: Ident,
    colon: syn::Token![:],
    ty: Box<Type>,
}

impl Parse for VarInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(VarInput {
            source: input.parse()?,
            comma: input.parse()?,
            mutability: input.parse()?,
            name: input.parse()?,
            colon: input.parse()?,
            ty: {
                match input.parse::<Type>()? {
                    Type::Group(group) => group.elem,
                    ty => Box::new(ty),
                }
            },
        })
    }
}

#[derive(Debug)]
pub enum VarSource {
    Option,
    Argument,
}

#[derive(Debug)]
pub enum VarType {
    Type(Box<Type>),
    Vec(Box<Type>),
    Slice(Box<Type>),
    MutSlice(Box<Type>),
    Option(Box<Type>),
}

impl VarType {
    #[allow(dead_code)]
    pub fn get_type(&self) -> &Type {
        match self {
            VarType::Type(ty) => ty.as_ref(),
            VarType::Vec(ty) => ty.as_ref(),
            VarType::Slice(ty) => ty.as_ref(),
            VarType::MutSlice(ty) => ty.as_ref(),
            VarType::Option(ty) => ty.as_ref(),
        }
    }
}

pub fn get_var_type(ty: &Type) -> VarType {
    match ty {
        Type::Path(_) => {
            if ty.is_vec() || ty.is_option() {
                let mut generics = ty.generic_arguments();
                assert_eq!(generics.len(), 1);

                if let GenericArgument::Type(generic_type) = generics.pop().unwrap() {
                    if ty.is_vec() {
                        VarType::Vec(Box::new(generic_type))
                    } else {
                        VarType::Option(Box::new(generic_type))
                    }
                } else {
                    unreachable!()
                }
            } else {
                VarType::Type(Box::new(ty.clone()))
            }
        }
        Type::Reference(type_ref) => {
            if let Type::Slice(type_slice) = type_ref.elem.as_ref() {
                if type_ref.mutability.is_some() {
                    VarType::MutSlice(type_slice.elem.clone())
                } else {
                    VarType::Slice(type_slice.elem.clone())
                }
            } else {
                VarType::Type(Box::new(ty.clone()))
            }
        }
        _ => VarType::Type(Box::new(ty.clone())),
    }
}
