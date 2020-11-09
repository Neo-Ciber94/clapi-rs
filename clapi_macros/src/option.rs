use crate::args::ArgData;
use proc_macro2::TokenStream;
use quote::*;
use crate::utils::to_str_literal_stream2;

/// Tokens for:
///
/// ```text
/// #[option(
///     name="value",
///     alias="v",
///     description="A description",
///     min=0,
///     max=100,
///     default="Hello World"
/// )]
/// ```
#[derive(Debug)]
pub struct OptionData {
    name: String,
    alias: Option<String>,
    description: Option<String>,
    args: Option<ArgData>,
}

impl OptionData {
    pub fn new(name: String) -> Self {
        OptionData {
            name,
            alias: None,
            description: None,
            args: None,
        }
    }

    pub fn set_alias(&mut self, alias: String){
        self.alias = Some(alias);
    }

    pub fn set_description(&mut self, description: String){
        self.description = Some(description);
    }

    pub fn set_args(&mut self, args: ArgData){
        self.args = Some(args);
    }

    pub fn expand(&self) -> TokenStream {
        // CommandOption::set_alias
        let alias = if let Some(s) = &self.alias{
            let tokens = to_str_literal_stream2(s).unwrap();
            quote!{ .set_alias(#tokens) }
        } else {
            quote!{}
        };

        // CommandOption::set_description
        let description = if let Some(s) = &self.description{
            let tokens = to_str_literal_stream2(s).unwrap();
            quote!{ .set_description(#tokens) }
        } else {
            quote!{}
        };

        // `CommandOption::set_required` is args have default values
        let required = match &self.args {
            Some(args) if !args.has_default_values() => {
                quote! { .set_required(true) }
            }
            _ => quote! {}
        };

        // CommandOption::set_args
        let args = if let Some(args) = &self.args{
            let tokens = args.expand();
            quote!{ .set_args(#tokens) }
        } else {
            quote!{}
        };

        let name = to_str_literal_stream2(&self.name).unwrap();

        quote!{
            clapi::option::CommandOption::new(#name)
            #alias
            #description
            #required
            #args
        }
    }
}

impl ToTokens for OptionData {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.expand().into_iter())
    }
}