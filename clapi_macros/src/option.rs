use crate::arg::ArgAttrData;
use crate::attr;
use crate::command::{is_option_bool_flag, FnArgData};
use crate::macro_attribute::Value;
use proc_macro2::{Span, TokenStream};
use quote::*;

/// Tokens for an `option` attribute.
///
/// ```text
/// #[command]
/// #[option(numbers, alias="n", description="Average", min=1, max=100, default=0)]
/// fn avg(numbers: Vec<i64>){
///     println!("{}", numbers.iter().sum::<i64>() / numbers.len() as i64);
/// }
/// ```
#[derive(Debug)]
pub struct OptionAttrData {
    name: String,
    alias: Option<String>,
    description: Option<String>,
    args: Option<ArgAttrData>,
}

impl OptionAttrData {
    pub fn new(name: String) -> Self {
        OptionAttrData {
            name,
            alias: None,
            description: None,
            args: None,
        }
    }

    pub fn from_arg_data(arg_data: FnArgData) -> Self {
        let mut option = OptionAttrData::new(arg_data.arg_name.clone());
        let mut args = ArgAttrData::from_arg_data(arg_data.clone().drop_attribute());

        if let Some(att) = &arg_data.attribute {
            for (key, value) in att {
                match key.as_str() {
                    attr::ARG => {
                        let arg_name = value
                            .clone()
                            .to_string_literal()
                            .expect("option `arg` must be a string literal");

                        args.set_name(arg_name);
                    }
                    attr::ALIAS => {
                        let alias = value
                            .clone()
                            .to_string_literal()
                            .expect("option `alias` must be a string literal");

                        option.set_alias(alias);
                    }
                    attr::DESCRIPTION => {
                        let description = value
                            .clone()
                            .to_string_literal()
                            .expect("option `description` must be a string literal");
                        option.set_description(description);
                    }
                    attr::MIN => {
                        let min = value
                            .clone()
                            .to_integer_literal::<usize>()
                            .expect("option `min` must be an integer literal");

                        args.set_min(min);
                    }
                    attr::MAX => {
                        let max = value
                            .clone()
                            .to_integer_literal::<usize>()
                            .expect("option `max` must be an integer literal");

                        args.set_max(max);
                    }
                    attr::DEFAULT => match value {
                        Value::Literal(lit) => args.set_default_values(vec![lit.clone()]),
                        Value::Array(array) => args.set_default_values(array.clone()),
                    },
                    _ => panic!("invalid `{}` key `{}`", att.path(), key),
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
            args.set_max(1);
        }

        option.set_args(args);
        option
    }

    pub fn set_alias(&mut self, alias: String) {
        self.alias = Some(alias);
    }

    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
    }

    pub fn set_args(&mut self, args: ArgAttrData) {
        self.args = Some(args);
    }

    pub fn expand(&self) -> TokenStream {
        // CommandOption::set_alias
        let alias = if let Some(s) = &self.alias {
            quote! { .alias(#s) }
        } else {
            quote! {}
        };

        // CommandOption::set_description
        let description = if let Some(s) = &self.description {
            quote! { .description(#s) }
        } else {
            quote! {}
        };

        // `CommandOption::set_required` is args have default values
        let required = match &self.args {
            Some(args) if !args.has_default_values() => {
                quote! { .required(true) }
            }
            _ => quote! {},
        };

        // CommandOption::set_args
        let args = if let Some(args) = &self.args {
            let tokens = args.expand();
            quote! { .arg(#tokens) }
        } else {
            quote! {}
        };

        let name = quote_expr!(self.name);

        quote! {
            clapi::CommandOption::new(#name)
            #alias
            #description
            #required
            #args
        }
    }
}

impl ToTokens for OptionAttrData {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.expand().into_iter())
    }
}
