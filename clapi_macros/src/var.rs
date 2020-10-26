use crate::{parse_to_str_stream2, IteratorExt};
use proc_macro2::TokenStream;
use quote::*;
use syn::export::ToTokens;
use syn::{GenericArgument, Pat, PatType, PathArguments, Type};

#[derive(Debug, Clone)]
pub struct LocalVar {
    pub name: String,
    pub ty: VarType,
    pub is_mut: bool,
    pub from_args: bool,
}

impl LocalVar {
    pub fn new(pat_type: PatType, from_args: bool) -> LocalVar {
        new_local_var(None, pat_type, from_args)
    }

    pub fn with_name(name: String, pat_type: PatType, from_args: bool) -> LocalVar {
        new_local_var(Some(name), pat_type, from_args)
    }

    pub fn expand(&self) -> TokenStream {
        let path = parse_to_str_stream2(&self.name);

        let mutability = if self.is_mut {
            quote! { mut }
        } else {
            quote! {}
        };

        let source = if self.from_args {
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
            let name = parse_to_str_stream2(&self.name);

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

impl ToTokens for LocalVar {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.expand().into_iter())
    }
}

#[derive(Debug, Clone)]
pub enum VarType {
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

fn new_local_var(name: Option<String>, pat_type: PatType, from_args: bool) -> LocalVar {
    let path = name.unwrap_or_else(|| pat_type.pat.to_token_stream().to_string());
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
                            name: path,
                            is_mut,
                            from_args,
                            ty: VarType::Vec(ty),
                        }
                    }
                    _ => panic!("invalid arg: `{}`", pat_type.to_token_stream().to_string()),
                }
            } else {
                LocalVar {
                    name: path,
                    is_mut,
                    from_args,
                    ty: VarType::Vec(pat_type.ty),
                }
            }
        }
        Type::Reference(type_ref) => {
            if type_ref.mutability.is_some() {
                LocalVar {
                    name: path,
                    is_mut,
                    from_args,
                    ty: VarType::MutSlice(type_ref.elem.clone()),
                }
            } else {
                LocalVar {
                    name: path,
                    is_mut,
                    from_args,
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
