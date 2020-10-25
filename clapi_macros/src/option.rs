use crate::args::ArgsTokens;
use proc_macro2::TokenStream;
use quote::*;
use crate::parse_to_str_stream2;

/// Tokens for:
///
/// ```ignore
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
pub struct OptionTokens {
    name: String,
    alias: Option<String>,
    description: Option<String>,
    args: Option<ArgsTokens>,
}

impl OptionTokens {
    pub fn new(name: String) -> Self {
        OptionTokens{
            name,
            alias: None,
            description: None,
            args: None
        }
    }

    pub fn set_alias(&mut self, alias: String){
        self.alias = Some(alias);
    }

    pub fn set_description(&mut self, description: String){
        self.description = Some(description);
    }

    pub fn set_args(&mut self, args: ArgsTokens){
        self.args = Some(args);
    }

    pub fn expand(&self) -> TokenStream {
        let alias = if let Some(s) = &self.alias{
            let tokens = parse_to_str_stream2(s);
            quote!{ .set_alias(#tokens) }
        } else {
            quote!{}
        };

        let description = if let Some(s) = &self.description{
            let tokens = parse_to_str_stream2(s);
            quote!{ .set_description(#tokens) }
        } else {
            quote!{}
        };

        let args = if let Some(args) = &self.args{
            let tokens = args.expand();
            quote!{ .set_args(#tokens) }
        } else {
            quote!{}
        };

        let name = parse_to_str_stream2(&self.name);

        quote!{
            clapi::option::CommandOption::new(#name)
            #alias
            #description
            #args
        }
    }
}

impl ToTokens for OptionTokens{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.expand().into_iter())
    }
}