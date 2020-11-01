use crate::{parse_to_str_stream2, IteratorExt, parse_to_stream2};
use proc_macro2::TokenStream;
use quote::*;
use syn::export::ToTokens;
use syn::{GenericArgument, Pat, PatType, Type, PathSegment, PathArguments};
use syn::spanned::Spanned;

#[derive(Debug, Clone)]
pub struct ArgLocalVar{
    name: String,
    is_mut: bool,
    source: LocalVarSource,
    ty: ArgType
}

impl ArgLocalVar {
    pub fn new(pat_type: PatType, source: LocalVarSource) -> ArgLocalVar{
        new_arg_local_var(pat_type, source)
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn arg_type(&self) -> &ArgType {
        &self.ty
    }

    pub fn expand(&self) -> TokenStream {
        let is_mut = if self.is_mut { quote! { mut }} else { quote! {} };
        let source = match self.source {
            LocalVarSource::Args(_) => self.get_args_source(),
            LocalVarSource::Opts => self.get_opts_source(),
        };

        let var_name = parse_to_stream2(&self.name);

        match self.ty {
            ArgType::Slice(_) => {
                let concat = format!("tmp_{}", var_name);
                let temp = syn::Ident::new(&concat, var_name.span());

                quote! {
                    let #is_mut #temp = #source ;
                    let #is_mut #var_name = #temp.as_slice() ;
                }
            }
            ArgType::MutSlice(_) => {
                let concat = format!("tmp_{}", var_name);
                let temp = syn::Ident::new(&concat, var_name.span());

                quote! {
                    let #is_mut #temp = #source ;
                    let #is_mut #var_name = #temp.as_mut_slice() ;
                }
            },
            _ => {
                quote! {
                    let #is_mut #var_name = #source ;
                }
            }
        }
    }

    fn get_opts_source(&self) -> TokenStream {
        let arg_name = parse_to_str_stream2(&self.name);

        match &self.ty {
            ArgType::Raw(ty) => {
                quote! { opts.get_args(#arg_name).unwrap().convert_at::<#ty>(0).unwrap() }
            }
            ArgType::Vec(ty) | ArgType::Slice(ty) | ArgType::MutSlice(ty) => {
                quote! { opts.get_args(#arg_name).unwrap().convert_all::<#ty>().unwrap() }
            }
            ArgType::Option(ty) => {
                quote! {
                    match options.len(){
                        0 => None,
                        _ => Some(opts.get_args(#arg_name).unwrap().convert_at::<#ty>(0).unwrap())
                    }
                }
            }
        }
    }

    fn get_args_source(&self) -> TokenStream {
        match &self.ty {
            ArgType::Raw(ty) => {
                if let LocalVarSource::Args(index) = self.source {
                    quote! { args.convert_at::<#ty>(#index).unwrap() }
                } else {
                    unreachable!()
                }
            }
            ArgType::Vec(ty) | ArgType::Slice(ty) | ArgType::MutSlice(ty) => {
                quote! { args.convert_all::<#ty>().unwrap() }
            }
            ArgType::Option(ty) => {
                quote! {
                    match args.values.len(){
                        0 => None,
                        _ => Some(args.convert_at::<#ty>(0).unwrap())
                    }
                }
            }
        }
    }
}

impl ToTokens for ArgLocalVar {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.expand().into_iter())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LocalVarSource {
    /// The value from the arguments.
    Args(usize),
    /// The value from the options.
    Opts
}

#[derive(Debug, Clone)]
pub enum ArgType{
    Raw(Box<Type>),
    Vec(Box<Type>),
    Slice(Box<Type>),
    MutSlice(Box<Type>),
    Option(Box<Type>)
}

impl ArgType {
    pub fn inner_type(&self) -> &Type{
        match self {
            ArgType::Raw(ty) => ty.as_ref(),
            ArgType::Vec(ty) => ty.as_ref(),
            ArgType::Slice(ty) => ty.as_ref(),
            ArgType::MutSlice(ty) => ty.as_ref(),
            ArgType::Option(ty) => ty.as_ref()
        }
    }

    pub fn is_raw(&self) -> bool{
        matches!(self, ArgType::Raw(_))
    }

    pub fn is_vec(&self) -> bool{
        matches!(self, ArgType::Vec(_))
    }

    pub fn is_slice(&self) -> bool{
        matches!(self, ArgType::Slice(_))
    }

    pub fn is_mut_slice(&self) -> bool{
        matches!(self, ArgType::MutSlice(_))
    }

    pub fn is_option(&self) -> bool{
        matches!(self, ArgType::Option(_))
    }

    pub fn is_array(&self) -> bool {
        self.is_vec() || self.is_slice() || self.is_mut_slice()
    }
}

enum OuterType {
    Vec, Option
}

fn new_arg_local_var(pat_type: PatType, source: LocalVarSource) -> ArgLocalVar {
    let name =  pat_type.pat.to_token_stream().to_string();
    let ty = get_arg_type(&pat_type);
    let is_mut = match pat_type.pat.as_ref() {
        Pat::Ident(ident) => ident.mutability.is_some(),
        _ => false,
    };

    ArgLocalVar { name, is_mut, source, ty, }
}

fn get_arg_type(pat_type: &PatType) -> ArgType {
    match pat_type.ty.as_ref() {
        Type::Path(type_path) => {
            let last_path = type_path.path.segments
                .last()
                .unwrap_or_else(|| panic!("invalid arg type: `{}`", pat_type.to_token_stream().to_string()));

            match last_path.ident.to_string() {
                ident if is_vec(ident.as_str()) => {
                    get_inner_type(pat_type, OuterType::Vec, last_path)
                },
                ident if is_option(ident.as_str()) => {
                    get_inner_type(pat_type, OuterType::Option, last_path)
                },
                _ => ArgType::Raw(pat_type.ty.clone())
            }
        }
        Type::Reference(ref_type) => {
            if let Type::Slice(array) = ref_type.elem.as_ref() {
                match ref_type.mutability {
                    Some(_) => ArgType::MutSlice(array.elem.clone()),
                    None => ArgType::Slice(array.elem.clone()),
                }
            } else {
                panic!("expected slice found reference: `{}`", ref_type.to_token_stream().to_string());
            }
        }
        _ => panic!(
            "invalid type: arg `{}`",
            pat_type.to_token_stream().to_string()
        ),
    }
}

fn get_inner_type(pat_type: &PatType, outer: OuterType, path_segment: &PathSegment) -> ArgType {
    match &path_segment.arguments {
        PathArguments::AngleBracketed(angle_bracketed) => {
            let generic_arg =
                angle_bracketed.args.iter().single().unwrap_or_else(|| {
                    panic!("multiple generics defined: `{}`",
                           pat_type.to_token_stream().to_string()
                    )
                });

            let ty = match generic_arg {
                GenericArgument::Type(ty) => Box::new(ty.clone()),
                _ => {
                    panic!("invalid arg: `{}`", pat_type.to_token_stream().to_string())
                }
            };

            match outer {
                OuterType::Vec => ArgType::Vec(ty.clone()),
                OuterType::Option => ArgType::Option(ty.clone())
            }
        }
        _ => panic!("invalid arg type: `{}`", pat_type.to_token_stream().to_string())
    }
}

fn is_vec(ident: &str) -> bool{
    match ident {
        "Vec" => true,
        "std::vec::Vec" => true,
        _ => false
    }
}

fn is_option(ident: &str) -> bool{
    match ident {
        "Option" => true,
        "std::option::Option" => true,
        "core::option::Option" => true,
        _ => false
    }
}