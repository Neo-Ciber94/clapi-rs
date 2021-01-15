use crate::arg::ArgAttrData;
use crate::attr;
use crate::command::{is_option_bool_flag, FnArgData};
use crate::macro_attribute::{Value, MacroAttribute};
use proc_macro2::{Span, TokenStream};
use quote::*;

/// Tokens for an `option` attribute.
///
/// ```text
/// #[command]
/// #[option(numbers,
///     alias="n",
///     description="Average",
///     hidden = false,
///     multiple = false,
///     min=1,
///     max=100,
///     default=0)]
/// fn avg(numbers: Vec<i64>){
///     println!("{}", numbers.iter().sum::<i64>() / numbers.len() as i64);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct OptionAttrData {
    name: String,
    attribute: Option<MacroAttribute>,
    alias: Option<String>,
    description: Option<String>,
    arg: Option<ArgAttrData>,
    is_hidden: Option<bool>,
    allow_multiple: Option<bool>,
}

impl OptionAttrData {
    pub fn new(name: String) -> Self {
        OptionAttrData {
            name,
            attribute: None,
            alias: None,
            description: None,
            arg: None,
            is_hidden: None,
            allow_multiple: None,
        }
    }

    pub fn from_arg_data(arg_data: FnArgData) -> Self {
        let mut option = OptionAttrData::new(arg_data.arg_name.clone());
        let mut args = ArgAttrData::from_arg_data(arg_data.clone());

        if let Some(att) = &arg_data.name_value {
            for (key, value) in att {
                match key.as_str() {
                    attr::ARG => {
                        let arg_name = value
                            .to_string_literal()
                            .expect("option `arg` must be a string literal");

                        args.set_name(arg_name);
                    }
                    attr::ALIAS => {
                        let alias = value
                            .to_string_literal()
                            .expect("option `alias` must be a string literal");

                        option.set_alias(alias);
                    }
                    attr::DESCRIPTION => {
                        let description = value
                            .to_string_literal()
                            .expect("option `description` must be a string literal");

                        option.set_description(description);
                    }
                    attr::MIN => {
                        let min = value
                            .to_integer_literal::<usize>()
                            .expect("option `min` must be an integer literal");

                        args.set_min(min);
                    }
                    attr::MAX => {
                        let max = value
                            .to_integer_literal::<usize>()
                            .expect("option `max` must be an integer literal");

                        args.set_max(max);
                    }
                    attr::HIDDEN => {
                        let is_hidden = value
                            .to_bool_literal()
                            .expect("option `hidden` must be a bool literal");

                        option.set_hidden(is_hidden);
                    }
                    attr::MULTIPLE => {
                        let allow_multiple = value
                            .to_bool_literal()
                            .expect("option `multiple` must be a bool literal");

                        option.set_multiple(allow_multiple);
                    },
                    attr::DEFAULT => match value {
                        Value::Literal(lit) => args.set_default_values(vec![lit.clone()]),
                        Value::Array(array) => args.set_default_values(array.clone()),
                    },
                    attr::VALUES => match value {
                        Value::Literal(lit) => args.set_valid_values(vec![lit.clone()]),
                        Value::Array(array) => args.set_valid_values(array.clone()),
                    },
                    _ => panic!("invalid `option` key `{}`", key),
                }
            }
        }

        // A function argument is considered an option bool flag if:
        // - Is bool type
        // - Don't contains `min`, `max` or `default`
        if is_option_bool_flag(&arg_data) {
            // An option bool behaves like the follow:
            // --flag=true      (true)
            // --flag=false     (false)
            // --flag           (true)
            // [no option]      (false)

            // Is needed to set `false` as default value
            // to allow the option to be marked as no `required`
            let lit = syn::LitBool {
                value: false,
                span: Span::call_site(),
            };
            args.set_default_values(vec![syn::Lit::Bool(lit)]);
            args.set_min(0);
            args.set_max(1); //#[option]
        }

        // Sets the attribute and the args
        option.attribute = arg_data.attribute;
        option.set_args(args);
        option
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn set_alias(&mut self, alias: String) {
        self.alias = Some(alias);
    }

    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
    }

    pub fn set_args(&mut self, args: ArgAttrData) {
        self.arg = Some(args);
    }

    pub fn set_hidden(&mut self, is_hidden: bool) {
        self.is_hidden = Some(is_hidden);
    }

    pub fn set_multiple(&mut self, allow_multiple: bool){
        self.allow_multiple = Some(allow_multiple);
    }

    pub fn expand(&self) -> TokenStream {
        // Option alias
        let alias = self.alias
            .as_ref()
            .map(|s| quote! { .alias(#s) });

        // Option description
        let description = self.description
            .as_ref()
            .map(|s| quote! { .description(#s) });

        // Option is required if `args` have default values
        let required = match &self.arg {
            Some(args) if !args.has_default_values() => {
                quote! { .required(true) }
            }
            _ => quote! {},
        };

        // Option argument
        let arg = self.arg
            .as_ref()
            .map(|arg| quote! { .arg(#arg) });

        // Option is hidden
        let is_hidden = self.is_hidden
            .as_ref()
            .map(|value| quote! { .hidden(#value)} );

        // Option allow multiple
        let allow_multiple = self.allow_multiple
            .as_ref()
            .map(|value| quote! { .multiple(#value)} );

        let name = quote_expr!(self.name);

        quote! {
            clapi::CommandOption::new(#name)
            #alias
            #description
            #required
            #is_hidden
            #allow_multiple
            #arg
        }
    }
}

impl ToTokens for OptionAttrData {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.expand().into_iter())
    }
}

impl Eq for OptionAttrData{}

impl PartialEq for OptionAttrData {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}