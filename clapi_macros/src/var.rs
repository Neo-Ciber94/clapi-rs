use proc_macro2::TokenStream;
use quote::*;
use syn::export::{ToTokens, Formatter};
use syn::{GenericArgument, Pat, PatType, Type, PathSegment, PathArguments};
use syn::spanned::Spanned;
use crate::{IteratorExt, TypeExtensions};
use syn::export::fmt::Display;

#[derive(Debug, Clone)]
pub struct ArgLocalVar{
    name: String,
    is_mut: bool,
    source: VarSource,
    ty: ArgumentType
}

impl ArgLocalVar {
    pub fn new(pat_type: PatType, source: VarSource) -> Self {
        new_arg_local_var(pat_type, source)
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn arg_type(&self) -> &ArgumentType {
        &self.ty
    }

    pub fn expand(&self) -> TokenStream {
        let var_name = self.name.parse::<TokenStream>().unwrap();
        let is_mut = if self.is_mut { quote! { mut }} else { quote! {} };
        let source = match &self.source {
            VarSource::Args(arg_name) => self.get_args_source(arg_name),
            VarSource::Opts(arg_name) => self.get_opts_source(arg_name),
            VarSource::OptBool => {
                let opt_name = quote_expr!(self.name);
                quote! { opts.contains(#opt_name) }
            }
        };

        match self.ty {
            ArgumentType::Slice(_) | ArgumentType::MutSlice(_) => {
                let concat = format!("tmp_{}", var_name);
                let temp = syn::Ident::new(&concat, var_name.span());
                let as_slice = match &self.ty {
                    ArgumentType::Slice(_) => quote! { .as_slice() },
                    ArgumentType::MutSlice(_) => quote! { .as_mut_slice() },
                    _ => unreachable!()
                };

                quote! {
                    let #is_mut #temp = #source ;
                    let #is_mut #var_name = #temp #as_slice ;
                }
            },
            _ => {
                quote! {
                    let #is_mut #var_name = #source ;
                }
            }
        }
    }

    fn get_opts_source(&self, arg_name: &str) -> TokenStream {
        let option_name = quote_expr!(self.name);
        let arg_name = quote_expr!(arg_name);

        match &self.ty {
            ArgumentType::Type(ty) => {
                quote! { opts.get(#option_name).unwrap().get_args().get(#arg_name).unwrap().convert::<#ty>()? }
            }
            ArgumentType::Vec(ty) | ArgumentType::Slice(ty) | ArgumentType::MutSlice(ty) => {
                quote! { opts.get(#option_name).unwrap().get_args().get(#arg_name).unwrap().convert_all::<#ty>()? }
            }
            ArgumentType::Option(ty) => {
                let option_arg = format_ident!("{}_arg", self.name);
                quote! {
                    {
                        let #option_arg = opts.get_args(#option_name).unwrap().get(#arg_name).unwrap();
                        match #option_arg.get_values().len(){
                            0 => None,
                            _ => Some(#option_arg.convert::<#ty>()?)
                        }
                    }
                }
            }
        }
    }

    fn get_args_source(&self, arg_name: &str) -> TokenStream {
        match &self.ty {
            ArgumentType::Type(ty) => {
                if let VarSource::Args(_) = &self.source {
                    quote! { args.get(#arg_name).unwrap().convert::<#ty>()? }
                } else {
                    unreachable!()
                }
            }
            ArgumentType::Vec(ty) | ArgumentType::Slice(ty) | ArgumentType::MutSlice(ty) => {
                quote! { args.get(#arg_name).unwrap().convert_all::<#ty>()? }
            }
            ArgumentType::Option(ty) => {
                let arg_temp = format_ident!("{}_temp", self.name);
                quote! {
                    {
                        let #arg_temp = args.get(#arg_name).unwrap();
                        match #arg_temp.get_values().len(){
                            0 => None,
                            _ => Some(#arg_temp.convert::<#ty>()?)
                        }
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

#[derive(Debug, Clone)]
pub enum VarSource {
    /// The value from the arguments.
    Args(String),
    /// The value from the options.
    Opts(String),
    /// The value from an option flag
    OptBool
}

#[derive(Debug, Clone)]
pub enum ArgumentType {
    Type(Box<Type>),
    Vec(Box<Type>),
    Slice(Box<Type>),
    MutSlice(Box<Type>),
    Option(Box<Type>)
}

impl ArgumentType {
    pub fn new(pat_type: &PatType) -> Self {
        get_arg_type(pat_type)
    }

    pub fn get_type(&self) -> &Type{
        match self {
            ArgumentType::Type(ty) => ty.as_ref(),
            ArgumentType::Vec(ty) => ty.as_ref(),
            ArgumentType::Slice(ty) => ty.as_ref(),
            ArgumentType::MutSlice(ty) => ty.as_ref(),
            ArgumentType::Option(ty) => ty.as_ref(),
        }
    }

    pub fn is_raw(&self) -> bool{
        matches!(self, ArgumentType::Type(_))
    }

    pub fn is_vec(&self) -> bool{
        matches!(self, ArgumentType::Vec(_))
    }

    pub fn is_slice(&self) -> bool{
        matches!(self, ArgumentType::Slice(_))
    }

    pub fn is_mut_slice(&self) -> bool{
        matches!(self, ArgumentType::MutSlice(_))
    }

    pub fn is_option(&self) -> bool{
        matches!(self, ArgumentType::Option(_))
    }
}

impl Display for ArgumentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_type().to_token_stream().to_string())
    }
}

enum OuterType {
    Vec, Option
}

fn new_arg_local_var(pat_type: PatType, source: VarSource) -> ArgLocalVar {
    let name =  pat_type.pat.to_token_stream().to_string();
    let ty = get_argument_type(&pat_type);
    let is_mut = match pat_type.pat.as_ref() {
        Pat::Ident(ident) => ident.mutability.is_some(),
        _ => false,
    };

    ArgLocalVar { name, is_mut, source, ty, }
}

fn get_arg_type(pat_type: &PatType) -> ArgumentType {
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
                _ => ArgumentType::Type(pat_type.ty.clone())
            }
        }
        Type::Reference(ref_type) => {
            if let Type::Slice(array) = ref_type.elem.as_ref() {
                match ref_type.mutability {
                    Some(_) => ArgumentType::MutSlice(array.elem.clone()),
                    None => ArgumentType::Slice(array.elem.clone()),
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

fn get_inner_type(pat_type: &PatType, outer: OuterType, path_segment: &PathSegment) -> ArgumentType {
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
                OuterType::Vec => ArgumentType::Vec(ty.clone()),
                OuterType::Option => ArgumentType::Option(ty.clone())
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

fn get_argument_type(pat_type: &PatType) -> ArgumentType {
    match pat_type.ty.as_ref() {
        Type::Path(_) => {
            return if pat_type.ty.is_vec() {
                ArgumentType::Vec(generic_type(pat_type))
            } else if pat_type.ty.is_option(){
                ArgumentType::Option(generic_type(pat_type))
            } else {
                ArgumentType::Type(pat_type.ty.clone())
            }
        },
        Type::Reference(type_ref) => {
            return if let Type::Slice(array) = type_ref.elem.as_ref() {
                match type_ref.mutability {
                    Some(_) => ArgumentType::MutSlice(array.elem.clone()),
                    None => ArgumentType::Slice(array.elem.clone()),
                }
            } else {
                panic!("expected slice found reference: `{}`", pat_type.to_token_stream().to_string());
            }
        },
        _ => {
            panic_invalid_argument_type(pat_type)
        }
    }
}

fn generic_type(pat_type: &PatType) -> Box<Type> {
    if let Type::Path(type_path) = pat_type.ty.as_ref() {
        let segment = type_path.path.segments
            .last()
            .unwrap_or_else(|| panic_invalid_argument_type(pat_type));

        if let PathArguments::AngleBracketed(angle_bracketed_generics) = &segment.arguments {
            let generic = angle_bracketed_generics.args
                .iter()
                .single()
                .unwrap_or_else(|| panic!("multiple generics defined: `{}`", pat_type.to_token_stream().to_string()));

            if let GenericArgument::Type(ty) = generic {
                return Box::new(ty.clone())
            }
        }
    }

    panic_invalid_argument_type(pat_type)
}

fn panic_invalid_argument_type(pat_type: &PatType) -> !{
    panic!("invalid argument type: `{}`", pat_type.to_token_stream().to_string());
}