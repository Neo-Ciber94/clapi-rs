#![allow(clippy::len_zero, clippy::redundant_closure, clippy::bool_comparison)]
use crate::arg::ArgAttrData;
use crate::macro_attribute::{MacroAttribute, NameValueAttribute};
use crate::option::OptionAttrData;
use crate::utils::NamePath;
use crate::var::{ArgLocalVar, ArgumentType};
use crate::TypeExt;
use proc_macro2::TokenStream;
use quote::*;
use std::path::PathBuf;
use syn::{AttrStyle, Attribute, AttributeArgs, Item, ItemFn, PatType, ReturnType, Stmt, Type};

/// Tokens for either `command` or `subcommand` attribute.
///
/// ```text
/// #[command(
/// description="Prints system time",
/// about="Gets the current system time in milliseconds",
/// version=0.1
/// )]
/// fn system_time(){
///     #[subcommand(description="Prints the current operative system", version="1.0.2")]
///     fn os(){
///         println!("{}", std::env::consts::OS);
///     }
///
///     println!("{}", std::time::SystemTime::now().elapsed().unwrap().as_millis());
/// }
/// ```
#[derive(Debug, Clone)]
pub struct CommandAttrData {
    fn_name: NamePath,
    name: Option<String>,
    attribute: NameValueAttribute,
    is_child: bool,
    version: Option<String>,
    description: Option<String>,
    usage: Option<StringSource>,
    help: Option<StringSource>,
    item_fn: Option<ItemFn>,
    children: Vec<CommandAttrData>,
    is_hidden: Option<bool>,
    options: Vec<OptionAttrData>,
    args: Vec<ArgAttrData>,
    vars: Vec<ArgLocalVar>,
    command_help: Option<NamePath>,
    command_usage: Option<NamePath>,
}

impl CommandAttrData {
    pub fn from_fn(args: AttributeArgs, func: ItemFn) -> Self {
        let name = func.sig.ident.to_string();
        let attr_data =
            NameValueAttribute::from_attribute_args(
                name.as_str(),
                args,
                AttrStyle::Outer
        ).expect("failed to parse `command` attribute");

        CommandAttrData::new_from_fn(attr_data, func, false, true, true)
    }

    pub fn from_path(args: AttributeArgs, func: ItemFn, path: PathBuf) -> Self {
        imp::command_from_path(args, func, path)
    }

    fn new(fn_name: NamePath, attribute: NameValueAttribute, is_child: bool) -> Self {
        CommandAttrData {
            fn_name,
            is_child,
            attribute,
            name: None,
            version: None,
            description: None,
            usage: None,
            help: None,
            item_fn: None,
            children: vec![],
            options: vec![],
            vars: vec![],
            args: vec![],
            is_hidden: None,
            command_help: None,
            command_usage: None,
        }
    }

    fn new_from_fn(
        name_value_attr: NameValueAttribute,
        item_fn: ItemFn,
        is_child: bool,
        get_subcommands: bool,
        get_help: bool,
    ) -> Self {
        if is_child {
            assert!(!get_help, "cannot `get_help` in a subcommand");
        }

        let name = item_fn.sig.ident.to_string();
        imp::command_from_fn_with_name(
            NamePath::new(name),
            name_value_attr,
            item_fn,
            is_child,
            get_subcommands,
            get_help,
        )
    }

    pub fn set_name(&mut self, name: String) {
        assert!(self.name.is_none(), "command `name` is already defined");
        assert!(!name.trim().is_empty(), "command `name` cannot be empty");
        assert!(
            name.trim().chars().all(|c| !c.is_whitespace()),
            "command `name` cannot contain whitespaces"
        );
        self.name = Some(name);
    }

    pub fn set_version(&mut self, version: String) {
        assert!(
            self.version.is_none(),
            "command `version` is already defined"
        );
        assert!(!version.trim().is_empty(), "`version` cannot be empty");
        self.version = Some(version);
    }

    pub fn set_description(&mut self, description: String) {
        assert!(
            self.description.is_none(),
            "command `description` is already defined"
        );
        self.description = Some(description);
    }

    pub fn set_usage(&mut self, usage: StringSource) {
        assert!(self.usage.is_none(), "command `usage` is already defined");
        self.usage = Some(usage);
    }

    pub fn set_help(&mut self, help: StringSource) {
        assert!(self.help.is_none(), "command `help` is already defined");
        self.help = Some(help);
    }

    pub fn set_child(&mut self, command: CommandAttrData) {
        assert!(command.is_child);
        if self.children.contains(&command) {
            panic!(
                "duplicated subcommand: `{}` in `{}`",
                command.fn_name.name(),
                self.fn_name.name()
            )
        }

        self.children.push(command);
    }

    pub fn set_hidden(&mut self, is_hidden: bool) {
        assert!(
            self.is_hidden.is_none(),
            "command `is_hidden` is already defined"
        );

        if is_hidden {
            assert!(self.is_child, "only subcommands can be hidden");
        }

        self.is_hidden = Some(is_hidden);
    }

    pub fn set_option(&mut self, option: OptionAttrData) {
        if self.options.contains(&option) {
            panic!(
                "duplicated option: `{}` in `{}`",
                option.name(),
                self.fn_name.name()
            )
        }

        self.options.push(option);
    }

    pub fn set_args(&mut self, args: ArgAttrData) {
        if self.args.contains(&args) {
            panic!(
                "duplicated arg: `{}` in `{}`",
                args.name(),
                self.fn_name.name()
            )
        }
        self.args.push(args);
    }

    pub fn set_var(&mut self, var: ArgLocalVar) {
        if self.vars.contains(&var) {
            panic!(
                "duplicated variable: `{}` in `{}`",
                var.name(),
                self.fn_name.name()
            )
        }
        self.vars.push(var);
    }

    pub fn set_item_fn(&mut self, item_fn: ItemFn) {
        self.item_fn = Some(item_fn);
    }

    pub fn set_command_help(&mut self, name_path: NamePath) {
        self.command_help = Some(name_path);
    }

    pub fn set_command_usage(&mut self, name_path: NamePath) {
        self.command_usage = Some(name_path);
    }

    pub fn get_mut_recursive(&mut self, name_path: &NamePath) -> Option<&mut CommandAttrData> {
        if &self.fn_name == name_path {
            return Some(self);
        }

        for child in &mut self.children {
            if let Some(command) = child.get_mut_recursive(name_path) {
                return Some(command);
            }
        }

        None
    }

    pub fn expand(&self) -> TokenStream {
        assert!(
            self.item_fn.is_some(),
            "`ItemFn` is not set for command `{}`",
            self.fn_name
        );

        // Command args
        let args = self
            .args
            .iter()
            .map(|tokens| quote! { .arg(#tokens) })
            .collect::<Vec<TokenStream>>();

        // Command options
        let options = self
            .options
            .iter()
            .map(|x| quote! { .option(#x)})
            .collect::<Vec<TokenStream>>();

        // Command children
        let children = self
            .children
            .iter()
            .map(|x| quote! { .subcommand(#x)})
            .collect::<Vec<TokenStream>>();

        // Command function variables
        let vars = self
            .vars
            .iter()
            .map(|x| quote! { #x })
            .collect::<Vec<TokenStream>>();

        // Command description
        let description = self
            .description
            .as_ref()
            .map(|s| quote! { .description(#s) });

        // Command hidden
        let hidden = self.is_hidden.as_ref().map(|s| quote! { .hidden(#s) });

        // Command usage
        let usage = self.usage.as_ref().map(|s| quote! { .usage(#s) });

        // Command help
        let help = self.help.as_ref().map(|s| quote! { .help(#s) });

        // Command version
        let version = self.version.as_ref().map(|s| quote! { .version(#s) });

        // Instantiate `Command` or `RootCommand`
        let mut command = match &self.name {
            Some(name) => {
                quote! { clapi::Command::new(#name) }
            }
            None => {
                if self.is_child {
                    let fn_name = quote_expr!(self.fn_name.name());
                    quote! { clapi::Command::new(#fn_name) }
                } else {
                    quote! { clapi::Command::root() }
                }
            }
        };

        // Command handler
        let handler = if contains_expressions(self.item_fn.as_ref().unwrap()) {
            // Get the function body
            let body = self.get_body(vars.as_slice());

            quote! {
                .handler(|opts, args|{
                    #body
                })
            }
        } else {
            /*
                We omit the handler if don't contains any expressions or locals, for example:

                EMPTY:
                    fn test() {}
                    fn test(){ const VALUE : I64 = 0; }

                NOT EMPTY:
                    fn test() { println!("HELLO WORLD"); }
                    fn test() { let value = 0; }
            */
            quote! {}
        };

        // Build the command
        command = quote! {
            #command
                #description
                #usage
                #hidden
                #help
                #version
                #(#args)*
                #(#options)*
                #(#children)*
                #handler
        };

        if self.is_child {
            command
        } else {
            let name = self.fn_name.name().parse::<TokenStream>().unwrap();
            let ret = &self.item_fn.as_ref().unwrap().sig.output;
            let attrs = &self.item_fn.as_ref().unwrap().attrs;
            let items = self.get_body_items();
            let error_handling = match ret {
                ReturnType::Type(_, ty) if is_clapi_result_type(ty) => quote! {},
                _ => quote! { .map_err(|e| e.exit()).unwrap(); },
            };
            let use_help = {
                if self.command_help.is_none() && self.command_usage.is_none() {
                    quote! { .use_default_help() }
                } else {
                    let command_help = self
                        .command_help
                        .as_ref()
                        .map(|n| n.to_string().parse::<TokenStream>().unwrap())
                        .unwrap_or_else(|| quote! { clapi::help::command_help });

                    let command_usage = self
                        .command_usage
                        .as_ref()
                        .map(|n| n.to_string().parse::<TokenStream>().unwrap())
                        .unwrap_or_else(|| quote! { clapi::help::command_usage });

                    quote! {
                       .use_help({
                            clapi::help::HelpSource {
                                help: #command_help,
                                usage: #command_usage
                            }
                       })
                    }
                }
            };

            // Emit the tokens to create the function with the `RootCommand`
            quote! {
                #(#attrs)*
                fn #name() #ret {
                    #(#items)*

                    let command = #command ;
                    clapi::CommandLine::new(command)
                        #use_help
                        .use_default_suggestions()
                        .parse_args()
                        #error_handling
                }
            }
        }
    }

    fn get_body(&self, vars: &[TokenStream]) -> TokenStream {
        let ret = &self.item_fn.as_ref().unwrap().sig.output;
        let error_handling = match ret {
            ReturnType::Type(_, ty) if is_clapi_result_type(ty) => quote! {},
            // If return type is not `Result` we need return `fn_name(args) ; Ok(())`
            _ => quote! { ; Ok(()) },
        };

        if self.is_child {
            let fn_name = self.fn_name.to_string().parse::<TokenStream>().unwrap();
            let inputs = self.vars.iter().map(|var| {
                let var_name = var.name().parse::<TokenStream>().unwrap();
                let is_ref = match var.arg_type() {
                    ArgumentType::Slice(ty) => {
                        if ty.mutability {
                            quote! { &mut }
                        } else {
                            quote! { & }
                        }
                    }
                    _ => quote! {},
                };

                quote! { #is_ref #var_name }
            });

            quote! {
                #(#vars)*
                #fn_name(#(#inputs,)*)
                #error_handling
            }
        } else {
            let statements = self.get_body_statements();

            quote! {
                #(#vars)*
                #(#statements)*
                #error_handling
            }
        }
    }

    fn get_body_statements(&self) -> Vec<TokenStream> {
        // locals, expressions, ...
        self.item_fn
            .as_ref()
            .unwrap()
            .block
            .stmts
            .iter()
            .filter(|s| !matches!(s, Stmt::Item(_)))
            .map(|s| s.to_token_stream())
            .collect()
    }

    fn get_body_items(&self) -> Vec<TokenStream> {
        fn contains_subcommand_attribute(item_fn: &ItemFn) -> bool {
            item_fn.attrs.iter().any(|attribute| {
                let path = crate::utils::path_to_string(&attribute.path);
                crate::consts::is_subcommand(&path)
            })
        }

        fn statement_to_tokens(stmt: Stmt) -> TokenStream {
            if let Stmt::Item(Item::Fn(ref item_fn)) = stmt {
                if contains_subcommand_attribute(item_fn) && !contains_expressions(item_fn) {
                    let mut item_fn = item_fn.clone();
                    crate::utils::insert_allow_dead_code_attribute(&mut item_fn);
                    return drop_command_attributes(item_fn).to_token_stream();
                }
            }

            stmt.to_token_stream()
        }

        // functions, struct, impl, const, statics, ...
        self.item_fn
            .as_ref()
            .unwrap()
            .block
            .stmts
            .clone()
            .into_iter()
            .filter(|s| matches!(s, Stmt::Item(_)))
            .map(statement_to_tokens)
            .collect()
    }
}

impl ToTokens for CommandAttrData {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.expand().into_iter())
    }
}

impl Eq for CommandAttrData {}

impl PartialEq for CommandAttrData {
    fn eq(&self, other: &Self) -> bool {
        self.fn_name == other.fn_name
    }
}

// Information about an function argument of a function decorated with a `clapi_macros` attribute
#[derive(Debug, Clone)]
pub struct FnArgData {
    // Name of the function argument
    pub arg_name: String,
    // The function argument `PatType` like: `x : i64`
    pub pat_type: PatType,
    // The macro attribute if any, this is solely used for debugging,
    // the actual information is from the `NameValueAttribute`
    pub attribute: Option<MacroAttribute>,
    // The name-values of the macro attribute
    pub name_value: Option<NameValueAttribute>,
    // If the function argument correspond to a command option.
    pub is_option: bool,
}

// Represents the source of the string data used.
#[derive(Debug, Clone)]
pub enum StringSource {
    // A string literal.
    String(String),
    // A function in the form: `fn() -> Into<String>`
    Fn(syn::ExprPath),
}

impl StringSource {
    // Creates a `StringSource::Fn` from a function path identifier
    pub fn from_fn_path(s: &str) -> Result<Self, syn::Error> {
        assert!(!s.trim().is_empty());
        let path: syn::ExprPath = syn::parse_str(s)?;
        Ok(StringSource::Fn(path))
    }
}

impl ToTokens for StringSource {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            StringSource::String(s) => {
                tokens.extend(s.into_token_stream());
            }
            StringSource::Fn(path) => tokens.extend(quote! { #path() }),
        }
    }
}

fn is_clapi_result_type(ty: &Type) -> bool {
    if ty.is_result() {
        return true;
    }

    matches!(
        ty.path().unwrap().as_str(),
        "clapi::Result" | "clapi::error::Result"
    )
}

/// Check the statements of the `ItemFn` and returns `true`
/// if there is not expression declared.
///
/// The next is considered empty:
/// ```text
/// fn main(){
///     static VALUE: i64 = 0;
/// }
/// ```
///
/// Where the next don't
/// ```text
/// fn main(){
///     let value = 0;
/// }
/// ```
pub fn contains_expressions(item_fn: &ItemFn) -> bool {
    use std::ops::Not;

    item_fn
        .block
        .stmts
        .iter()
        .all(|stmt| matches!(stmt, Stmt::Item(_)))
        .not()
}

/// Remove all the `clapi` macro attributes like `command`, `subcommand`, `option` and `arg`
/// from a `ItemFn`.
pub fn drop_command_attributes(mut item_fn: ItemFn) -> ItemFn {
    item_fn.attrs = item_fn
        .attrs
        .iter()
        .filter(|att| {
            let path = att.path.to_token_stream().to_string();
            !crate::consts::is_clapi_attribute(&path)
        })
        .cloned()
        .collect::<Vec<Attribute>>();

    item_fn
}

/// Checks if a function argument can be considered an option bool flag like: `--enable`
///
/// In the next example, `enable` is considered an option bool flag when passing: `--enable`
/// the parameter enable will have the value of `true` and `false` if absent.
///
/// ```text
/// #[command]
/// fn main(enable: bool){}
/// ```
pub fn is_option_bool_flag(fn_arg: &FnArgData) -> bool {
    // Only `option`s can be bool flags
    if !fn_arg.is_option {
        return false;
    }

    // Of course, only bool can be an option bool flag
    if !fn_arg.pat_type.ty.is_bool() {
        return false;
    }

    if let Some(attribute) = &fn_arg.name_value {
        let min = attribute
            .get(crate::consts::MIN)
            .map(|v| {
                v.to_integer_literal::<usize>()
                    .expect("`min` must be a integer literal")
            })
            .unwrap_or(0);

        let max = attribute
            .get(crate::consts::MAX)
            .map(|v| {
                v.to_integer_literal::<usize>()
                    .expect("`max` must be a integer literal")
            })
            .unwrap_or(1);

        let default = attribute
            .get(crate::consts::DEFAULT)
            .map(|v| {
                v.to_bool_literal()
                    .expect("`default` must be a bool literal")
            })
            .unwrap_or(false);

        // Is an option bool flag is is boolean type and: min = 0, max = 1, default = false
        min == 0 && max == 1 && default == false
    } else {
        true
    }
}

mod imp {
    use crate::arg::ArgAttrData;
    use crate::command::{
        drop_command_attributes, is_option_bool_flag, CommandAttrData, FnArgData, StringSource,
    };
    use crate::macro_attribute::{MacroAttribute, NameValueAttribute};
    use crate::option::OptionAttrData;
    use crate::query::QueryItem;
    use crate::utils::{path_to_string, NamePath};
    use crate::var::{ArgLocalVar, VarSource};
    use crate::{consts, AttrQuery};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, Ordering};
    use syn::{AttrStyle, Attribute, AttributeArgs, File, FnArg, Item, ItemFn, PatType, Stmt};
    use quote::ToTokens;

    // Constructs a new `CommandAttrData` from a `ItemFn`
    pub fn command_from_fn_with_name(
        name: NamePath,
        name_value_attr: NameValueAttribute,
        item_fn: ItemFn,
        is_child: bool,
        get_subcommands: bool,
        get_help: bool,
    ) -> CommandAttrData {
        let mut command = CommandAttrData::new(name, name_value_attr.clone(), is_child);

        for (key, value) in &name_value_attr {
            match key.as_str() {
                crate::consts::NAME => {
                    let name = value
                        .to_string_literal()
                        .expect("`name` must be a string literal");

                    command.set_name(name);
                }
                crate::consts::PARENT if is_child => { /*Parent is handle bellow*/ }
                crate::consts::DESCRIPTION => {
                    let description = value
                        .to_string_literal()
                        .expect("`description` must be a string literal");

                    command.set_description(description);
                }
                crate::consts::HIDDEN => {
                    let hidden = value
                        .to_bool_literal()
                        .expect("`hidden` must be a bool literal");

                    command.set_hidden(hidden);
                }
                crate::consts::VERSION => {
                    assert!(
                        value.is_integer() || value.is_float() || value.is_string(),
                        "`version` must be an integer, float or string literal"
                    );
                    command.set_version(value.parse_literal::<String>().unwrap());
                }
                crate::consts::USAGE => {
                    let usage = value
                        .to_string_literal()
                        .expect("`usage` must be a string literal");

                    command.set_usage(StringSource::String(usage));
                }
                crate::consts::HELP => {
                    let help = value
                        .to_string_literal()
                        .expect("`help` must be a string literal");

                    command.set_help(StringSource::String(help));
                }
                crate::consts::WITH_USAGE => {
                    let expr = value
                        .to_string_literal()
                        .expect("`with_usage` must be a string literal");

                    let path = path_to_relative(&expr, &command.fn_name)
                        .unwrap_or_else(|| panic!("invalid expression: {}", expr))
                        .to_string();

                    let s = StringSource::from_fn_path(path.as_str())
                        .unwrap_or_else(|_|{
                            panic!("invalid expression for `with_usage` expected function path, but was {}", expr)
                        });
                    command.set_usage(s);
                }
                crate::consts::WITH_HELP => {
                    let expr = value
                        .to_string_literal()
                        .expect("`with_usage` must be a string literal");

                    let path = path_to_relative(&expr, &command.fn_name)
                        .unwrap_or_else(|| panic!("invalid expression: {}", expr))
                        .to_string();

                    let s = StringSource::from_fn_path(path.as_str()).unwrap_or_else(|_| {
                        panic!(
                            "invalid expression for `with_help` expected function path, but was {}",
                            expr
                        )
                    });

                    command.set_help(s);
                }
                _ => panic!("invalid `{}` key: `{}`", name_value_attr.path(), key),
            }
        }

        let fn_args = get_fn_args(&item_fn);
        let arg_count = fn_args.iter().filter(|f| !f.is_option).count();

        // Pass function arguments in order
        for fn_arg in &fn_args {
            if fn_arg.is_option {
                let source = if is_option_bool_flag(fn_arg) {
                    VarSource::OptBool
                } else {
                    VarSource::Opts(fn_arg.arg_name.clone())
                };
                command.set_var(ArgLocalVar::new(fn_arg.pat_type.clone(), source));
            } else {
                command.set_var(ArgLocalVar::new(
                    fn_arg.pat_type.clone(),
                    VarSource::Args(fn_arg.arg_name.clone()),
                ));
            }
        }

        // Add args
        if arg_count > 0 {
            for fn_arg in fn_args.iter().filter(|f| !f.is_option) {
                command.set_args(ArgAttrData::from_arg_data(fn_arg.clone()));
            }
        }

        // Add options
        for fn_arg in fn_args.into_iter().filter(|n| n.is_option) {
            let option = OptionAttrData::from_arg_data(fn_arg);
            command.set_option(option);
        }

        // Add children
        if get_subcommands {
            let mut subcommands = Vec::new();
            for (attribute, item_fn) in get_subcommands_from_fn(&item_fn) {
                let subcommand =
                    CommandAttrData::new_from_fn(attribute.clone(), item_fn, true, true, false);

                subcommands.push((subcommand, attribute.clone()));
            }

            while let Some((subcommand, attribute)) = subcommands.pop() {
                if attribute.contains(crate::consts::PARENT) {
                    let literal = attribute
                        .get(crate::consts::PARENT)
                        .unwrap()
                        .to_string_literal()
                        .expect("`parent` must be a string literal");

                    // If attribute was: #[subcommand(parent="")]
                    assert!(
                        literal.trim().len() > 0,
                        "`parent` was empty in `fn {}`",
                        subcommand.fn_name.name()
                    );

                    // Converts the path in `literal` to a relative to `subcommand` module
                    let name_path =
                        path_to_relative(&literal, &subcommand.fn_name).unwrap_or_else(|| {
                            panic!(
                                "cannot find parent command `{}` for `{}`",
                                literal,
                                subcommand.fn_name.name()
                            )
                        });

                    if let Some(parent) = command.get_mut_recursive(&name_path) {
                        parent.set_child(subcommand);
                    } else {
                        let mut found = false;

                        for (c, _) in &mut subcommands {
                            if let Some(parent) = c.get_mut_recursive(&name_path) {
                                parent.set_child(subcommand.clone());
                                found = true;
                                break;
                            }
                        }

                        if !found {
                            panic!(
                                "cannot find parent subcommand: `{}` for `{}`",
                                name_path, subcommand.fn_name
                            );
                        }
                    }
                } else {
                    command.set_child(subcommand);
                }
            }
        }

        // Gets the command help/usage
        if get_help {
            // Helper function
            fn find_inner_decorate_item_fn(
                item_fn: &ItemFn,
                attribute_name: &str,
            ) -> Option<ItemFn> {
                let mut result = item_fn
                    .block
                    .stmts
                    .iter()
                    .filter_map(|stmt| {
                        if let Stmt::Item(Item::Fn(item_fn)) = stmt {
                            if item_fn.contains_attribute(attribute_name) {
                                Some(item_fn)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .cloned()
                    .collect::<Vec<ItemFn>>();

                if result.len() > 1 {
                    panic!("multiple `#[{}]` defined", attribute_name);
                } else {
                    result.pop().map(|x| x)
                }
            }

            // Finds the `command_help` if any in the body of the function
            if let Some(command_help) = find_inner_decorate_item_fn(&item_fn, consts::COMMAND_HELP)
            {
                assert!(command.command_help.is_none(), "multiple #[{}] defined", consts::COMMAND_HELP);
                command.set_command_help(NamePath::new(command_help.sig.ident.to_string()));
            }

            // Finds the `command_usage` if any in the body of the function
            if let Some(command_usage) =
                find_inner_decorate_item_fn(&item_fn, consts::COMMAND_USAGE)
            {
                assert!(command.command_usage.is_none(), "multiple #[{}] defined", consts::COMMAND_USAGE);
                command.set_command_usage(NamePath::new(command_usage.sig.ident.to_string()));
            }
        }

        command.set_item_fn(drop_command_attributes(item_fn));
        command
    }

    // Get all the inner `fn` subcommands from the given `ItemFn`
    fn get_subcommands_from_fn(item_fn: &ItemFn) -> Vec<(NameValueAttribute, ItemFn)> {
        let mut ret = Vec::new();

        for stmt in &item_fn.block.stmts {
            if let Stmt::Item(Item::Fn(item_fn)) = stmt {
                let subcommands = item_fn
                    .attrs
                    .iter()
                    .filter(|att| crate::consts::is_subcommand(path_to_string(&att.path).as_str()))
                    .cloned()
                    .collect::<Vec<Attribute>>();

                if subcommands.len() > 0 {
                    assert_eq!(
                        subcommands.len(),
                        1,
                        "multiples `subcommand` attributes defined in `{}`",
                        item_fn.sig.ident.to_string()
                    );

                    let mut inner_fn = item_fn.clone();

                    let name_value_attr = if let Some(index) =
                        inner_fn.attrs.iter().position(|att| {
                            crate::consts::is_subcommand(path_to_string(&att.path).as_str())
                        }) {
                        MacroAttribute::new(inner_fn.attrs.swap_remove(index))
                            .into_name_values()
                            .unwrap()
                    } else {
                        unreachable!()
                    };

                    ret.push((name_value_attr, inner_fn))
                }
            }
        }

        ret
    }

    // Gets all the `FnArgData` from the given `ItemFn`
    fn get_fn_args(item_fn: &ItemFn) -> Vec<FnArgData> {
        fn get_fn_arg_ident_name(fn_arg: &FnArg) -> (String, PatType) {
            if let FnArg::Typed(pat_type) = &fn_arg {
                if let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                    return (pat_ident.ident.to_string(), pat_type.clone());
                }
            }

            panic!(
                "`{}` is not a valid function arg",
                fn_arg.to_token_stream().to_string()
            );
        }

        let mut ret = Vec::new();

        // We takes all the attributes that are `[arg(...)]` or `[option(...)]`
        let attributes = item_fn
            .attrs
            .iter()
            .cloned()
            .map(MacroAttribute::new)
            .filter(|attribute| {
                crate::consts::is_option(attribute.path())
                    || crate::consts::is_arg(attribute.path())
            })
            .map(split_attr_path_and_name_values)
            .collect::<Vec<(String, MacroAttribute, NameValueAttribute)>>();

        // Get all the function params and the name of the params
        let fn_args = item_fn
            .sig
            .inputs
            .iter()
            .map(|f| get_fn_arg_ident_name(f))
            .collect::<Vec<(String, PatType)>>();

        // Look for duplicated fn arg attribute declaration for example:
        // #[arg(x)]
        // #[option(x)]
        for index in 0..attributes.len() {
            let (name, attribute, _) = &attributes[index];
            if attributes[(index + 1)..]
                .iter()
                .any(|(arg, ..)| arg == name)
            {
                panic!(
                    "function argument `{}` is already used in `{}`",
                    name, attribute
                );
            }
        }

        // Check the argument declared in the `option` or `arg` exists in the function
        for (path, _, _) in &attributes {
            if !fn_args.iter().any(|(arg_name, _)| arg_name == path) {
                panic!(
                    "argument `{}` is no defined in `fn {}`",
                    path, item_fn.sig.ident
                );
            }
        }

        for (arg_name, pat_type) in fn_args {
            let (attribute, name_value) = if let Some(data) =
                attributes.iter().find_map(|(path, attribute, name_value)| {
                    if path == &arg_name {
                        Some((attribute.clone(), name_value.clone()))
                    } else {
                        None
                    }
                }) {
                (Some(data.0), Some(data.1))
            } else {
                (None, None)
            };

            let is_option = attribute
                .as_ref()
                .map(|attribute| crate::consts::is_option(attribute.path()))
                .unwrap_or(true);

            ret.push(FnArgData {
                arg_name,
                pat_type,
                attribute,
                name_value,
                is_option,
            });
        }

        ret
    }

    // Takes a `MacroAttribute` and returns its path, self and this name values
    fn split_attr_path_and_name_values(
        attribute: MacroAttribute,
    ) -> (String, MacroAttribute, NameValueAttribute) {
        let name = attribute.get(0)
            .cloned()
            .unwrap_or_else(|| panic!("the first element in `{}` must be the argument name, but was empty", attribute))
            .into_path()
            .unwrap_or_else(|| {
                panic!("first element in `{}` must be a path like: `#[{}(value, ...)]` where `value` is the name of the function argument", attribute, attribute.path())
            });

        let name_values = if attribute.len() == 1 {
            NameValueAttribute::empty(attribute.path().to_owned(), AttrStyle::Outer)
        } else {
            let meta_items = attribute[1..].to_vec();
            NameValueAttribute::new(attribute.path(), meta_items, AttrStyle::Outer).unwrap()
        };

        (name, attribute, name_values)
    }

    // Implementation of `CommandAttrData::from_path`
    pub fn command_from_path(
        args: AttributeArgs,
        item_fn: ItemFn,
        root_path: PathBuf,
    ) -> CommandAttrData {
        // Ensure there is only 1 `#[command]` defined
        static IS_DEFINED: AtomicBool = AtomicBool::new(false);
        if let Ok(true) =
            IS_DEFINED.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        {
            panic!(
                "multiple `command` entry points defined: `{}`",
                item_fn.sig.ident
            );
        }

        let src = std::fs::read_to_string(&root_path).unwrap();
        let file = syn::parse_file(&src).unwrap();
        assert_is_top_free_function(&file, &item_fn);

        // The attribute of the root command
        let attribute =
            NameValueAttribute::from_attribute_args(crate::consts::COMMAND, args, AttrStyle::Outer)
                .unwrap();

        // The root command which is the command decorated with `#[command]`
        let mut root = CommandAttrData::new_from_fn(attribute, item_fn, false, true, true);

        // Gets all the subcommands searching in all the modules
        let mut subcommands = get_subcommands_data(&root_path);

        // Finds and set the `command_help` if any
        if let Some(command_help) = find_decorated_item_fn(&root_path, consts::COMMAND_HELP) {
            root.set_command_help(command_help);
        }

        // Finds and set the `command_usage` if any
        if let Some(command_usage) = find_decorated_item_fn(&root_path, consts::COMMAND_USAGE) {
            root.set_command_usage(command_usage);
        }

        while let Some((subcommand, _, attribute)) = subcommands.pop() {
            if attribute.contains(crate::consts::PARENT) {
                let literal = attribute
                    .get(crate::consts::PARENT)
                    .unwrap()
                    .to_string_literal()
                    .expect("`parent` must be a `string` literal");

                // If attribute was: #[subcommand(parent="")]
                assert!(
                    literal.trim().len() > 0,
                    "`parent` was empty in `fn {}`",
                    subcommand.fn_name.name()
                );

                // Converts the path in `literal` to a relative to `subcommand` module
                let parent_name =
                    path_to_relative(&literal, &subcommand.fn_name).unwrap_or_else(|| {
                        panic!(
                            "cannot find parent command `{}` for `{}`",
                            literal,
                            subcommand.fn_name.name()
                        )
                    });

                if parent_name == subcommand.fn_name {
                    panic!(
                        "self reference command parent in `{}`",
                        subcommand.attribute
                    );
                }

                if let Some(parent) = find_command_recursive(&mut root, &parent_name) {
                    parent.set_child(subcommand);
                } else {
                    let mut found = false;
                    for (c, ..) in &mut subcommands {
                        if let Some(parent) = find_command_recursive(c, &parent_name) {
                            parent.set_child(subcommand.clone());
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        let parent_name = attribute
                            .get(crate::consts::PARENT)
                            .unwrap()
                            .to_string_literal()
                            .unwrap();

                        panic!(
                            "cannot find parent command: `{}` for `{}`",
                            parent_name,
                            subcommand.fn_name.name()
                        );
                    }
                }
            } else {
                root.set_child(subcommand);
            }
        }

        root
    }

    // Find a `ItemFn` with the specified attribute and gets its `NamePath`
    fn find_decorated_item_fn(root_path: &Path, attribute_name: &str) -> Option<NamePath> {
        let mut result = crate::query::find_items(root_path, true, true, |item| {
            if let Item::Fn(item_fn) = item {
                item_fn.contains_attribute(attribute_name)
            } else {
                false
            }
        });

        if result.len() > 1 {
            panic!("multiple `#[{}]` defined", attribute_name)
        }

        result.pop().map(|x| x.name_path)
    }

    // Finds the command with the given `NamePath` starting from the command to its children.
    fn find_command_recursive<'a>(
        command: &'a mut CommandAttrData,
        name: &'a NamePath,
    ) -> Option<&'a mut CommandAttrData> {
        if &command.fn_name == name {
            return Some(command);
        }

        for child in &mut command.children {
            if let Some(c) = find_command_recursive(child, name) {
                return Some(c);
            }
        }

        None
    }

    // Converts the `path` to a relative of `root`.
    // If root is `module::utils::math` and `path` is `super::get_id`
    // the return is `module::utils::get_id`
    fn path_to_relative(path: &str, root: &NamePath) -> Option<NamePath> {
        // `self` is ignore because: `module::self::self` == `module`
        let mut parent_path = path
            .split("::")
            .filter(|s| s != &"self")
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();

        // If the path starts like: `crate::` we remove `crate`.
        if let Some(first) = parent_path.first() {
            if first == "crate" {
                parent_path.remove(0);
            }
        }

        // The base path of the item, we ignore its name
        let mut ret = Vec::from(root.item_path());

        // Navigate from the `current` to the `parent`
        for s in parent_path {
            if s == "super" {
                if ret.pop().is_none() {
                    break;
                }
            } else {
                ret.push(s);
            }
        }

        if ret.is_empty() {
            return None;
        }

        Some(NamePath::from_path(ret))
    }

    // Starting from the given path and going through it's modules find all the `#[subcommand]`s.
    fn get_subcommands_data(
        root_path: &Path,
    ) -> Vec<(CommandAttrData, PathBuf, NameValueAttribute)> {
        let query_data = get_subcommands_item_fn(root_path);
        let mut subcommands = Vec::new();

        for QueryItem {
            path,
            name_path,
            item: (item_fn, attr),
            ..
        } in query_data
        {
            let src = std::fs::read_to_string(&path).unwrap();
            let file = syn::parse_file(&src).unwrap();
            assert_is_top_free_function(&file, &item_fn);

            let command =
                command_from_fn_with_name(name_path, attr.clone(), item_fn, true, false, false);

            subcommands.push((command, path, attr));
        }

        subcommands
    }

    // Helper function for `get_subcommands_data` this returns all the `#[subcommand]`s from the given path
    // and it's modules but contained in a `QueryItem<(ItemFn, NameValueAttribute)`
    fn get_subcommands_item_fn(root_path: &Path) -> Vec<QueryItem<(ItemFn, NameValueAttribute)>> {
        fn if_subcommand_to_name_value(attribute: &Attribute) -> Option<NameValueAttribute> {
            if consts::is_subcommand(&path_to_string(&attribute.path)) {
                Some(
                    MacroAttribute::new(attribute.clone())
                        .into_name_values()
                        .unwrap(),
                )
            } else {
                None
            }
        }

        crate::query::find_map_items(root_path, true, true, |item| {
            if let Item::Fn(item_fn) = item {
                if let Some(attribute) = item_fn.attrs.iter().find_map(if_subcommand_to_name_value)
                {
                    return Some((item_fn.clone(), attribute));
                }
            }

            None
        })
    }

    fn assert_is_top_free_function(file: &File, item_fn: &ItemFn) {
        fn eq_item_fn(left: &ItemFn, right: &ItemFn) -> bool {
            left.block == right.block && left.sig == right.sig && left.vis == right.vis
        }

        for item in &file.items {
            if let Item::Fn(cur_fn) = item {
                if eq_item_fn(cur_fn, item_fn) {
                    return;
                }
            }
        }

        panic!("`{}` is not a top free function", item_fn.sig.ident);
    }
}
