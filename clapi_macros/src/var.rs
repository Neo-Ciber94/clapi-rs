use crate::{parse_to_str_stream2, parse_to_stream2, IteratorExt};
use clapi::args::Arguments;
use clapi::option::Options;
use proc_macro2::TokenStream;
use quote::*;
use syn::export::ToTokens;
use syn::{GenericArgument, Pat, PatType, PathArguments, Type};

#[derive(Debug, Clone)]
pub struct LocalVar {
    path: String,
    ty: VarType,
    is_mut: bool,
    is_args: bool,
}

impl LocalVar {
    pub fn new(pat_type: PatType, is_args: bool) -> LocalVar {
        new_local_var(pat_type, is_args)
    }

    pub fn expand(&self) -> TokenStream {
        let path = parse_to_str_stream2(&self.path);

        let mutability = if self.is_mut {
            quote! { mut }
        } else {
            quote! {}
        };

        let source = if self.is_args {
            match &self.ty {
                VarType::Single(t) => {
                    quote! { args.convert::<#t>().unwrap() }
                }
                VarType::Vec(t) => {
                    quote! { args.convert_all::<#t>().unwrap() }
                }
                VarType::Slice(t) => {
                    quote! { args.convert_all::<#t>().unwrap().as_slice() }
                }
                VarType::MutSlice(t) => {
                    quote! { args.convert_all::<#t>().unwrap().as_mut_slice() }
                }
            }
        } else {
            let name = parse_to_str_stream2(&self.path);

            match &self.ty {
                VarType::Single(t) => {
                    quote! { opts.get_args(#name).unwrap().convert::<#t>().unwrap() }
                }
                VarType::Vec(t) => {
                    quote! { opts.get_args(#name).unwrap().convert_all::<#t>().unwrap() }
                }
                VarType::Slice(t) => {
                    quote! { opts.get_args(#name).unwrap().convert_all::<#t>().unwrap().as_slice() }
                }
                VarType::MutSlice(t) => {
                    quote! { opts.get_args(#name).unwrap().convert_all::<#t>().unwrap().as_mut_slice() }
                }
            }
        };

        quote! {
            let #mutability #path = #source;
        }
    }
}

impl ToTokens for LocalVar{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.expand().into_iter())
    }
}

#[derive(Debug, Clone)]
enum VarType {
    Single(Box<Type>),
    Vec(Box<Type>),
    Slice(Box<Type>),
    MutSlice(Box<Type>),
}

impl VarType {
    pub fn is_single(&self) -> bool {
        match self {
            VarType::Single(_) => true,
            _ => false,
        }
    }

    pub fn is_vec(&self) -> bool {
        match self {
            VarType::Vec(_) => true,
            _ => false,
        }
    }

    pub fn is_slice(&self) -> bool {
        match self {
            VarType::Slice(_) => true,
            _ => false,
        }
    }

    pub fn is_mut_slice(&self) -> bool {
        match self {
            VarType::MutSlice(_) => true,
            _ => false,
        }
    }

    pub fn inner(&self) -> &Type {
        match self {
            VarType::Single(ty) => ty,
            VarType::Vec(ty) => ty,
            VarType::Slice(ty) => ty,
            VarType::MutSlice(ty) => ty,
        }
    }
}

fn new_local_var(pat_type: PatType, is_args: bool) -> LocalVar {
    let path = pat_type.pat.to_token_stream().to_string();
    let is_mut = match pat_type.pat.as_ref() {
        Pat::Ident(ident) => ident.mutability.is_some(),
        _ => false,
    };

    match pat_type.ty.as_ref() {
        Type::Path(type_path) => {
            let last_path = type_path.path.segments.last().expect("invalid arg type");

            if last_path.ident.to_string() == "Vec" {
                match &last_path.arguments {
                    PathArguments::AngleBracketed(angle_bracketed) => {
                        let generic_arg =
                            angle_bracketed.args.iter().single().unwrap_or_else(|| {
                                panic!(
                                    "multiple generics defined: `{}`",
                                    pat_type.to_token_stream().to_string()
                                )
                            });

                        let ty = match generic_arg {
                            GenericArgument::Type(ty) => Box::new(ty.clone()),
                            _ => {
                                panic!("invalid arg: `{}`", pat_type.to_token_stream().to_string())
                            }
                        };

                        LocalVar {
                            path,
                            is_mut,
                            is_args,
                            ty: VarType::Vec(ty),
                        }
                    }
                    _ => panic!("invalid arg: `{}`", pat_type.to_token_stream().to_string()),
                }
            } else {
                LocalVar {
                    path,
                    is_mut,
                    is_args,
                    ty: VarType::Vec(pat_type.ty),
                }
            }
        }
        Type::Reference(type_ref) => {
            if type_ref.mutability.is_some() {
                LocalVar {
                    path,
                    is_mut,
                    is_args,
                    ty: VarType::MutSlice(type_ref.elem.clone()),
                }
            } else {
                LocalVar {
                    path,
                    is_mut,
                    is_args,
                    ty: VarType::Slice(type_ref.elem.clone()),
                }
            }
        }
        _ => panic!(
            "invalid arg type: `{}`",
            pat_type.to_token_stream().to_string()
        ),
    }
}
