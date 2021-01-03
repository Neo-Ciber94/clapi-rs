use crate::TypeExtensions;
use proc_macro2::TokenStream;
use quote::*;
use syn::export::fmt::Display;
use syn::export::{Formatter, ToTokens};
use syn::spanned::Spanned;
use syn::{GenericArgument, Pat, PatType, Type};

#[derive(Debug, Clone)]
pub struct ArgLocalVar {
    var_name: String,
    is_mut: bool,
    source: VarSource,
    ty: ArgumentType,
}

impl ArgLocalVar {
    pub fn new(pat_type: PatType, source: VarSource) -> Self {
        new_arg_local_var(pat_type, source)
    }

    pub fn name(&self) -> &str {
        self.var_name.as_str()
    }

    pub fn arg_type(&self) -> &ArgumentType {
        &self.ty
    }

    pub fn expand(&self) -> TokenStream {
        let var_name = self.var_name.parse::<TokenStream>().unwrap();
        let is_mut = if self.is_mut {
            quote! { mut }
        } else {
            quote! {}
        };
        let source = match &self.source {
            VarSource::Args(arg_name) => self.get_args_source(arg_name),
            VarSource::Opts(arg_name) => self.get_opts_source(arg_name),
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
                let option_name = quote_expr!(self.var_name);
                quote! {
                    match opts.get(#option_name) {
                        Some(option) => {
                            match option.get_arg() {
                                Some(arg) if arg.get_values().len() > 0 => arg.convert::<bool>()?,
                                Some(_) | None => true,
                            }
                        },
                        None => false
                    }
                }
            }
        };

        match self.ty {
            ArgumentType::Slice(_) | ArgumentType::MutSlice(_) => {
                let concat = format!("tmp_{}", var_name);
                let temp = syn::Ident::new(&concat, var_name.span());
                let as_slice = match &self.ty {
                    ArgumentType::Slice(_) => quote! { .as_slice() },
                    ArgumentType::MutSlice(_) => quote! { .as_mut_slice() },
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
        let option_name = quote_expr!(self.var_name);
        let arg_name = quote_expr!(arg_name);

        match &self.ty {
            ArgumentType::Type(ty) => {
                quote! { opts.get(#option_name).unwrap().get_args().get(#arg_name).unwrap().convert::<#ty>()? }
            }
            ArgumentType::Vec(ty) | ArgumentType::Slice(ty) | ArgumentType::MutSlice(ty) => {
                quote! { opts.get(#option_name).unwrap().get_args().get(#arg_name).unwrap().convert_all::<#ty>()? }
            }
            ArgumentType::Option(ty) => {
                let option_arg = format_ident!("{}_arg", self.var_name);
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
                let arg_temp = format_ident!("{}_temp", self.var_name);
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
    Slice(Box<Type>),
    MutSlice(Box<Type>),
    Option(Box<Type>),
}

impl ArgumentType {
    pub fn new(pat_type: &PatType) -> Self {
        get_argument_type(pat_type)
    }

    pub fn get_type(&self) -> &Type {
        match self {
            ArgumentType::Type(ty) => ty.as_ref(),
            ArgumentType::Vec(ty) => ty.as_ref(),
            ArgumentType::Slice(ty) => ty.as_ref(),
            ArgumentType::MutSlice(ty) => ty.as_ref(),
            ArgumentType::Option(ty) => ty.as_ref(),
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
        matches!(self, ArgumentType::MutSlice(_))
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

fn new_arg_local_var(pat_type: PatType, source: VarSource) -> ArgLocalVar {
    let name = pat_type.pat.to_token_stream().to_string();
    let ty = get_argument_type(&pat_type);
    let is_mut = match pat_type.pat.as_ref() {
        Pat::Ident(ident) => ident.mutability.is_some(),
        _ => false,
    };

    ArgLocalVar {
        var_name: name,
        is_mut,
        source,
        ty,
    }
}

fn get_argument_type(pat_type: &PatType) -> ArgumentType {
    match pat_type.ty.as_ref() {
        Type::Path(_) => {
            return if pat_type.ty.is_vec() {
                ArgumentType::Vec(generic_type(pat_type))
            } else if pat_type.ty.is_option() {
                ArgumentType::Option(generic_type(pat_type))
            } else {
                ArgumentType::Type(pat_type.ty.clone())
            }
        }
        Type::Reference(type_ref) => {
            return if let Type::Slice(array) = type_ref.elem.as_ref() {
                match type_ref.mutability {
                    Some(_) => ArgumentType::MutSlice(array.elem.clone()),
                    None => ArgumentType::Slice(array.elem.clone()),
                }
            } else {
                panic!(
                    "expected slice found reference: `{}`",
                    pat_type.to_token_stream().to_string()
                );
            }
        }
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
        return Box::new(ty.clone());
    } else {
        panic_invalid_argument_type(pat_type)
    }

    // if let Type::Path(type_path) = pat_type.ty.as_ref() {
    //     let segment = type_path.path.segments
    //         .last()
    //         .unwrap_or_else(|| panic_invalid_argument_type(pat_type));
    //
    //     if let PathArguments::AngleBracketed(angle_bracketed_generics) = &segment.arguments {
    //         let generic = angle_bracketed_generics.args
    //             .iter()
    //             .single()
    //             .unwrap_or_else(|| panic!("multiple generics defined: `{}`", pat_type.to_token_stream().to_string()));
    //
    //         if let GenericArgument::Type(ty) = generic {
    //             return Box::new(ty.clone())
    //         }
    //     }
    // }
}

fn panic_invalid_argument_type(pat_type: &PatType) -> ! {
    panic!(
        "invalid argument type: `{}`",
        pat_type.to_token_stream().to_string()
    );
}
