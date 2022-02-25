use crate::TypeExt;
use proc_macro2::TokenStream;
use quote::*;
use std::fmt::{Display, Formatter};
use syn::spanned::Spanned;
use syn::{Expr, GenericArgument, Pat, PatType, Type};

#[derive(Debug, Clone)]
pub struct ArgLocalVar {
    pub(crate) var_name: String,
    pub(crate) name: Option<String>,
    is_mut: bool,
    source: VarSource,
    ty: ArgumentType,
}

impl ArgLocalVar {
    pub fn new(pat_type: PatType, source: VarSource, name: Option<String>) -> Self {
        new_arg_local_var(pat_type, source, name)
    }

    pub fn var_name(&self) -> &str {
        self.var_name.as_str()
    }

    pub fn arg_type(&self) -> &ArgumentType {
        &self.ty
    }

    pub fn expand(&self) -> TokenStream {
        let var_name = self.var_name.as_str().parse::<TokenStream>().unwrap();
        let normalized_var_name = self
            .name
            .as_deref()
            .unwrap_or(&self.var_name)
            .trim_start_matches("r#");

        let is_mut = if self.is_mut {
            quote! { mut }
        } else {
            quote! {}
        };
        let source = match &self.source {
            VarSource::Args(arg_name) => {
                self.get_args_source(self.name.as_deref().unwrap_or(arg_name))
            }
            VarSource::Opts(arg_name) => {
                self.get_opts_source(self.name.as_deref().unwrap_or(arg_name))
            }
            VarSource::OptBool => {
                // Handles an option `bool` flag, which returns `true`
                // if the option exists or if the passed value is `true` otherwise `false`.
                //
                // Example:
                // #[command]
                // fn main(enable: bool) {}
                //
                // The parameter `enable` will takes the next value:
                // - `true` : If passing `--enable`
                // - `true` : If passing `--enable=true`
                // - `false`: If passing `--enable=false`
                // - `false`: If passing nothing
                let option_name = quote_expr!(normalized_var_name);
                quote! {
                    match opts.get(#option_name) {
                        None => false,
                        Some(option) => {
                            let arg = option.get_arg().unwrap();
                            match arg.convert::<bool>() {
                                Ok(v) => v,
                                Err(e) if e.kind() == &clapi::ErrorKind::InvalidArgumentCount => true,
                                Err(e) => return Err(e)
                            }
                        },
                    }
                }
            }
        };

        match self.ty {
            ArgumentType::Slice(_) => {
                let concat = format!("tmp_{}", normalized_var_name);
                let temp = syn::Ident::new(&concat, var_name.span());
                let as_slice = match &self.ty {
                    ArgumentType::Slice(slice) => {
                        if slice.mutability {
                            quote! { .as_mut_slice() }
                        } else {
                            quote! { .as_slice() }
                        }
                    }
                    _ => unreachable!(),
                };

                quote! {
                    let #is_mut #temp = #source ;
                    let #is_mut #var_name = #temp #as_slice ;
                }
            }
            _ => {
                quote! {
                    let #is_mut #var_name = #source ;
                }
            }
        }
    }

    fn get_opts_source(&self, arg_name: &str) -> TokenStream {
        let option_name = quote_expr!(self.var_name.as_str().trim_start_matches("r#"));
        let arg_name = quote_expr!(arg_name.trim_start_matches("r#"));

        match &self.ty {
            ArgumentType::Type(ty) => {
                quote! { opts.get(#option_name).unwrap().get_args().get(#arg_name).unwrap().convert::<#ty>()? }
            }
            ArgumentType::Vec(ty) => {
                quote! { opts.get(#option_name).unwrap().get_args().get(#arg_name).unwrap().convert_all::<#ty>()? }
            }
            ArgumentType::Slice(slice) => {
                let ty = &slice.ty;
                quote! { opts.get(#option_name).unwrap().get_args().get(#arg_name).unwrap().convert_all::<#ty>()? }
            }
            ArgumentType::Option(ty) => {
                quote! {
                    {
                        match opts.get_args(#arg_name)
                            .map(|args| args.get(#arg_name)).flatten() {
                            Some(arg) => {
                                match arg.get_values().len() {
                                    0 => None,
                                    _ => Some(arg.convert::<#ty>()?)
                                }
                            },
                            _ => None
                        }
                    }
                }
            }
            ArgumentType::Array(array) => {
                let ty = &array.ty;
                let len = &array.len;
                quote! {
                    {
                        let temp = opts.get(#option_name)
                            .unwrap()
                            .get_args()
                            .get(#arg_name)
                            .unwrap()
                            .convert_all<#ty>()?;

                        std::convert::TryInto::<[#ty; #len]>::try_into(temp).unwrap()
                    }
                }
            }
        }
    }

    fn get_args_source(&self, arg_name: &str) -> TokenStream {
        let normalized_name = arg_name.trim_start_matches("r#");

        match &self.ty {
            ArgumentType::Type(ty) => {
                if let VarSource::Args(_) = &self.source {
                    quote! { args.get(#normalized_name).unwrap().convert::<#ty>()? }
                } else {
                    unreachable!()
                }
            }
            ArgumentType::Vec(ty) => {
                quote! { args.get(#normalized_name).unwrap().convert_all::<#ty>()? }
            }
            ArgumentType::Slice(slice) => {
                let ty = &slice.ty;
                quote! { args.get(#normalized_name).unwrap().convert_all::<#ty>()? }
            }
            ArgumentType::Option(ty) => {
                let arg_temp = format_ident!("{}_temp", self.var_name);
                quote! {
                    {
                        let #arg_temp = args.get(#normalized_name).unwrap();
                        match #arg_temp.get_values().len(){
                            0 => None,
                            _ => Some(#arg_temp.convert::<#ty>()?)
                        }
                    }
                }
            }
            ArgumentType::Array(array) => {
                let ty = &array.ty;
                let len = &array.len;
                quote! {
                    {
                        let temp = args.get(#normalized_name).unwrap().convert_all::<#ty>()?;
                        std::convert::TryInto::<[#ty; #len]>::try_into(temp).unwrap()
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

impl Eq for ArgLocalVar {}

impl PartialEq for ArgLocalVar {
    fn eq(&self, other: &Self) -> bool {
        self.var_name == other.var_name
    }
}

#[derive(Debug, Clone)]
pub enum VarSource {
    /// The value from the arguments.
    Args(String),
    /// The value from the options.
    Opts(String),
    /// The value from an option flag
    OptBool,
}

#[derive(Debug, Clone)]
pub enum ArgumentType {
    Type(Box<Type>),
    Vec(Box<Type>),
    Option(Box<Type>),
    Slice(SliceType),
    Array(ArrayType),
}

#[derive(Debug, Clone)]
pub struct SliceType {
    pub ty: Box<Type>,
    pub mutability: bool,
}

#[derive(Debug, Clone)]
pub struct ArrayType {
    pub ty: Box<Type>,
    pub len: usize,
}

impl ArgumentType {
    pub fn new(pat_type: &PatType) -> Self {
        get_argument_type(pat_type)
    }

    pub fn get_type(&self) -> &Type {
        match self {
            ArgumentType::Type(ty) => ty.as_ref(),
            ArgumentType::Vec(ty) => ty.as_ref(),
            ArgumentType::Option(ty) => ty.as_ref(),
            ArgumentType::Slice(slice) => slice.ty.as_ref(),
            ArgumentType::Array(array) => array.ty.as_ref(),
        }
    }

    pub fn is_raw(&self) -> bool {
        matches!(self, ArgumentType::Type(_))
    }

    pub fn is_vec(&self) -> bool {
        matches!(self, ArgumentType::Vec(_))
    }

    pub fn is_slice(&self) -> bool {
        matches!(self, ArgumentType::Slice(_))
    }

    pub fn is_mut_slice(&self) -> bool {
        match self {
            ArgumentType::Slice(slice) => slice.mutability,
            _ => false,
        }
    }

    pub fn is_array(&self) -> bool {
        matches!(self, ArgumentType::Array(_))
    }

    pub fn is_option(&self) -> bool {
        matches!(self, ArgumentType::Option(_))
    }
}

impl Display for ArgumentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_type().to_token_stream().to_string())
    }
}

fn new_arg_local_var(pat_type: PatType, source: VarSource, name: Option<String>) -> ArgLocalVar {
    let var_name = pat_type.pat.to_token_stream().to_string();
    let ty = get_argument_type(&pat_type);
    let is_mut = match pat_type.pat.as_ref() {
        Pat::Ident(ident) => ident.mutability.is_some(),
        _ => false,
    };

    ArgLocalVar {
        var_name,
        name,
        is_mut,
        source,
        ty,
    }
}

fn get_argument_type(pat_type: &PatType) -> ArgumentType {
    match pat_type.ty.as_ref() {
        Type::Path(_) => {
            if pat_type.ty.is_vec() {
                ArgumentType::Vec(generic_type(pat_type))
            } else if pat_type.ty.is_option() {
                ArgumentType::Option(generic_type(pat_type))
            } else {
                ArgumentType::Type(pat_type.ty.clone())
            }
        }
        Type::Reference(type_ref) => {
            if let Type::Slice(array) = type_ref.elem.as_ref() {
                ArgumentType::Slice(SliceType {
                    ty: array.elem.clone(),
                    mutability: type_ref.mutability.is_some(),
                })
            } else {
                panic!(
                    "expected slice found reference: `{}`",
                    pat_type.to_token_stream().to_string()
                );
            }
        }
        Type::Array(type_array) => ArgumentType::Array(ArrayType {
            ty: type_array.elem.clone(),
            len: {
                if let Expr::Lit(expr) = &type_array.len {
                    if let syn::Lit::Int(int) = &expr.lit {
                        int.base10_parse::<usize>().unwrap()
                    } else {
                        panic!(
                            "array len must be a literal: `{}`",
                            pat_type.to_token_stream().to_string()
                        )
                    }
                } else {
                    panic!(
                        "array len must be a literal: `{}`",
                        pat_type.to_token_stream().to_string()
                    )
                }
            },
        }),
        _ => panic_invalid_argument_type(pat_type),
    }
}

fn generic_type(pat_type: &PatType) -> Box<Type> {
    let mut generic_arguments = pat_type.ty.generic_arguments();
    assert_eq!(
        generic_arguments.len(),
        1,
        "multiple generics defined: `{}`",
        pat_type.to_token_stream().to_string()
    );

    if let GenericArgument::Type(ty) = generic_arguments.pop().unwrap() {
        Box::new(ty)
    } else {
        panic_invalid_argument_type(pat_type)
    }
}

fn panic_invalid_argument_type(pat_type: &PatType) -> ! {
    panic!(
        "invalid argument type: `{}`",
        pat_type.to_token_stream().to_string()
    );
}
