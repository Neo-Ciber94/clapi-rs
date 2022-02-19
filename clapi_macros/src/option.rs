use crate::arg::ArgAttrData;
use crate::command::{is_option_bool_flag, FnArgData};
use crate::consts;
use crate::macro_attribute::{MacroAttribute, Value};
use proc_macro2::TokenStream;
use quote::*;
use syn::Lit;

/// Tokens for an `option` attribute.
///
/// ```text
/// #[command]
/// #[option(numbers,
///     alias="n",
///     description="Average",
///     hidden = false,
///     multiple = false,
///     global = false,
///     flag=false,
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
    is_global: Option<bool>,
    allow_multiple: Option<bool>,
    requires_assign: Option<bool>,
    is_flag: bool,
}

impl OptionAttrData {
    pub fn new(name: String) -> Self {
        assert!(!name.trim().is_empty(), "option `name` cannot be empty");
        assert!(
            name.trim().chars().all(|c| !c.is_whitespace()),
            "option `name` cannot contains whitespaces"
        );

        OptionAttrData {
            name,
            attribute: None,
            alias: None,
            description: None,
            arg: None,
            is_hidden: None,
            allow_multiple: None,
            requires_assign: None,
            is_global: None,
            is_flag: false,
        }
    }

    pub fn from_arg_data(arg_data: FnArgData) -> Self {
        let mut option = OptionAttrData::new(arg_data.arg_name.clone());
        let mut arg = ArgAttrData::from_arg_data(arg_data.clone());

        if let Some(att) = &arg_data.name_value {
            for (key, value) in att {
                match key.as_str() {
                    consts::NAME => {
                        let name = value
                            .to_string_literal()
                            .expect("option `name` must be a string literal");

                        option.set_name(name);
                    }
                    consts::ARG => {
                        let arg_name = value
                            .to_string_literal()
                            .expect("option `arg` must be a string literal");

                        arg.set_name(arg_name);
                    }
                    consts::ALIAS => {
                        let alias = value
                            .to_string_literal()
                            .expect("option `alias` must be a string literal");

                        option.set_alias(alias);
                    }
                    consts::DESCRIPTION => {
                        let description = value
                            .to_string_literal()
                            .expect("option `description` must be a string literal");

                        option.set_description(description);
                    }
                    consts::MIN => {
                        let min = value
                            .to_integer_literal::<usize>()
                            .expect("option `min` must be an integer literal");

                        arg.set_min(min);
                    }
                    consts::MAX => {
                        let max = value
                            .to_integer_literal::<usize>()
                            .expect("option `max` must be an integer literal");

                        arg.set_max(max);
                    }
                    consts::HIDDEN => {
                        let is_hidden = value
                            .to_bool_literal()
                            .expect("option `hidden` must be a bool literal");

                        option.set_hidden(is_hidden);
                    }
                    consts::MULTIPLE => {
                        let allow_multiple = value
                            .to_bool_literal()
                            .expect("option `multiple` must be a bool literal");

                        option.set_multiple(allow_multiple);
                    }
                    consts::REQUIRES_ASSIGN => {
                        let requires_assign = value
                            .to_bool_literal()
                            .expect("option `requires_assign` must be a bool literal");

                        option.set_requires_assign(requires_assign);
                    }
                    consts::ERROR => {
                        let error = value
                            .to_string_literal()
                            .expect("option `error` must be a string literal");

                        arg.set_validation_error(error);
                    }
                    consts::DEFAULT => match value {
                        Value::Literal(lit) => arg.set_default_values(vec![lit.clone()]),
                        Value::Array(array) => arg.set_default_values(array.clone() as Vec<Lit>),
                    },
                    consts::VALUES => match value {
                        Value::Literal(lit) => arg.set_valid_values(vec![lit.clone()]),
                        Value::Array(array) => arg.set_valid_values(array.clone() as Vec<Lit>),
                    },
                    consts::FLAG => {
                        // Just type checking
                        // This is used by `command.rs#is_option_bool_flag`
                        value
                            .to_bool_literal()
                            .expect("option `flag` must be a bool literal");
                    }
                    consts::GLOBAL => {
                        let global = value
                            .to_bool_literal()
                            .expect("option `global` must be a bool literal");

                        option.set_global(global);
                    }
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

            option.is_flag = true;
            arg.set_min(0);
            arg.set_max(1); //#[option]
        }

        // Sets the attribute and the args
        option.attribute = arg_data.attribute;
        option.set_args(arg);
        option
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn set_name(&mut self, name: String) {
        assert!(!name.trim().is_empty(), "option `name` cannot be empty");
        assert!(
            name.trim().chars().all(|c| !c.is_whitespace()),
            "option `name` cannot contains whitespaces"
        );

        self.name = name;
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

    pub fn set_multiple(&mut self, allow_multiple: bool) {
        self.allow_multiple = Some(allow_multiple);
    }

    pub fn set_requires_assign(&mut self, requires_assign: bool) {
        self.requires_assign = Some(requires_assign);
    }

    pub fn set_global(&mut self, global: bool) {
        self.is_global = Some(global);
    }

    pub fn expand(&self) -> TokenStream {
        // Option alias
        let alias = self.alias.as_ref().map(|s| quote! { .alias(#s) });

        // Option description
        let description = self
            .description
            .as_ref()
            .map(|s| quote! { .description(#s) });

        // Option is required if `args` don't have default values and is not a bool flag
        let required = match &self.arg {
            Some(args) if !args.has_default_values() && !self.is_flag => {
                quote! { .required(true) }
            }
            _ => quote! {},
        };

        // Option argument
        let arg = self.arg.as_ref().map(|arg| quote! { .arg(#arg) });

        // Option is hidden
        let is_hidden = self
            .is_hidden
            .as_ref()
            .map(|value| quote! { .hidden(#value) });

        // Option allow multiple
        let allow_multiple = self
            .allow_multiple
            .as_ref()
            .map(|value| quote! { .multiple(#value) });

        // Option requires assign
        let requires_assign = self
            .requires_assign
            .as_ref()
            .map(|value| quote! { .requires_assign(#value) });

        let is_global = self
            .is_global
            .as_ref()
            .map(|value| quote! { .global(#value) });

        let name = quote_expr!(self.name.as_str().trim_start_matches("r#"));

        quote! {
            clapi::CommandOption::new(#name)
            #alias
            #description
            #required
            #is_hidden
            #allow_multiple
            #requires_assign
            #is_global
            #arg
        }
    }
}

impl ToTokens for OptionAttrData {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.expand().into_iter())
    }
}

impl Eq for OptionAttrData {}

impl PartialEq for OptionAttrData {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
