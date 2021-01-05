#![allow(clippy::len_zero, clippy::redundant_closure)]
use std::path::PathBuf;
use proc_macro2::TokenStream;
use quote::*;
use syn::export::ToTokens;
use syn::{AttrStyle, Attribute, AttributeArgs, Item, ItemFn, PatType, ReturnType, Stmt, Type, ItemStatic};
use crate::arg::ArgAttrData;
use crate::macro_attribute::{NameValueAttribute, MacroAttribute};
use crate::option::OptionAttrData;
use crate::var::{ArgLocalVar, ArgumentType};
use crate::TypeExtensions;
use crate::utils::NamePath;

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
    attribute: NameValueAttribute,
    is_child: bool,
    version: Option<String>,
    description: Option<String>,
    about: Option<String>,
    item_fn: Option<ItemFn>,
    children: Vec<CommandAttrData>,
    options: Vec<OptionAttrData>,
    args: Vec<ArgAttrData>,
    vars: Vec<ArgLocalVar>,
    help: Option<ItemStatic>
}

impl CommandAttrData {
    fn new(name: NamePath, attribute: NameValueAttribute, is_child: bool) -> Self {
        CommandAttrData {
            fn_name: name,
            is_child,
            attribute,
            version: None,
            description: None,
            about: None,
            item_fn: None,
            children: vec![],
            options: vec![],
            vars: vec![],
            args: vec![],
            help: None
        }
    }

    pub fn from_fn(args: AttributeArgs, func: ItemFn) -> Self {
        let name = func.sig.ident.to_string();
        let attr_data = NameValueAttribute::from_attribute_args(name.as_str(), args, AttrStyle::Outer).unwrap();
        imp::command_from_fn(attr_data, func, false, true, true)
    }

    pub fn from_path(args: AttributeArgs, func: ItemFn, path: PathBuf) -> Self {
        imp::command_from_path(args, func, path)
    }

    pub fn set_version(&mut self, version: String) {
        assert!(!version.is_empty(), "command version cannot be empty");
        self.version = Some(version);
    }

    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
    }

    pub fn set_about(&mut self, about: String) {
        self.about = Some(about);
    }

    pub fn set_child(&mut self, command: CommandAttrData) {
        assert!(command.is_child);
        if self.children.contains(&command) {
            panic!("duplicated subcommand: `{}` in `{}`", command.fn_name.name(), self.fn_name.name())
        }

        self.children.push(command);
    }

    pub fn set_option(&mut self, option: OptionAttrData) {
        if self.options.contains(&option) {
            panic!("duplicated option: `{}` in `{}`", option.name(), self.fn_name.name())
        }

        self.options.push(option);
    }

    pub fn set_args(&mut self, args: ArgAttrData) {
        if self.args.contains(&args) {
            panic!("duplicated arg: `{}` in `{}`", args.name(), self.fn_name.name())
        }
        self.args.push(args);
    }

    pub fn set_var(&mut self, var: ArgLocalVar) {
        if self.vars.contains(&var) {
            panic!("duplicated variable: `{}` in `{}`", var.name(), self.fn_name.name())
        }
        self.vars.push(var);
    }

    pub fn set_item_fn(&mut self, item_fn: ItemFn) {
        self.item_fn = Some(item_fn);
    }

    pub fn set_help(&mut self, help: ItemStatic) {
        assert!(!self.is_child);
        self.help = Some(help)
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

        // Command version
        let version = self
            .version
            .as_ref()
            .map(|_| {
                quote! { .option(clapi::CommandOption::new("version").alias("v")) }
            })
            .unwrap_or_else(|| quote! {});

        // Command description
        let description = self
            .description
            .as_ref()
            .map(|s| quote! { #s })
            .map(|tokens| quote! { .description(#tokens)})
            .unwrap_or_else(|| quote! {});

        // Command help
        let about = self
            .about
            .as_ref()
            .map(|s| quote! { #s })
            .map(|tokens| quote! { .about(#tokens)})
            .unwrap_or_else(|| quote! {});

        // Get the function body
        let body = self.get_body(vars.as_slice());

        // Instantiate `Command` or `RootCommand`
        let mut command = if self.is_child {
            let command_name = quote_expr!(self.fn_name.name());
            quote! { clapi::Command::new(#command_name) }
        } else {
            quote! { clapi::Command::root() }
        };

        // Show version
        let show_version = self
            .version
            .as_ref()
            .map(|s| {
                quote! {
                    if opts.contains("version"){
                        println!("{} {}", clapi::current_filename(), #s);
                        return Ok(());
                    }
                }
            })
            .unwrap_or_else(|| quote! {});

        //println!("contains expressions {}: {}", self.fn_name, self.contains_expressions());
        // Command handler
        let handler = if contains_expressions(self.item_fn.as_ref().unwrap()) {
            quote!{
                .handler(|opts, args|{
                    #show_version
                    #body
                })
            }
        } else {
            // We omit the handler if don't contains any expressions or locals, for example:
            //
            // EMPTY:
            //      fn test() {}
            //      fn test(){ const VALUE : I64 = 0; }
            //
            // NOT EMPTY:
            //      fn test() { println!("HELLO WORLD"); }
            //      fn test() { let value = 0; }
            if show_version.is_empty() {
                quote!{}
            } else {
                quote!{
                    .handler(|opts, args|{
                        #show_version
                        Err(clapi::Error::from(clapi::ErrorKind::FallthroughHelp))
                    })
                }
            }
        };

        // Build the command
        command = quote! {
            #command
                #description
                #about
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
            let help = if self.help.is_some() {
                let help_body = self.get_help();
                quote!{
                    .set_help({ #help_body })
                }
            } else {
                quote! { .use_default_help() }
            };
            let error_handling = match ret {
                ReturnType::Type(_, ty) if is_clapi_result_type(ty) => quote! {},
                _ => quote! { .expect("an error occurred"); },
            };

            // Emit the tokens to create the function with the `RootCommand`
            quote! {
                #(#attrs)*
                fn #name() #ret {
                    #(#items)*

                    let command = #command ;
                    clapi::CommandLine::new(command)
                        #help
                        .use_default_suggestions()
                        .run()
                        #error_handling
                }
            }
        }
    }

    fn get_help(&self) -> TokenStream {
        let help = &self.help.as_ref().unwrap().ident;

        quote! {
            struct __Help;
            impl clapi::help::Help for __Help {
                #[inline]
                fn help(&self, buf: &mut clapi::help::Buffer, context: &clapi::Context, command: &clapi::Command) -> std::fmt::Result {
                    #help.help(buf, context, command)
                }

                #[inline]
                fn usage(&self, buf: &mut clapi::help::Buffer, context: &clapi::Context, command: &clapi::Command) -> std::fmt::Result {
                    #help.usage(buf, context, command)
                }

                #[inline]
                fn kind(&self) -> clapi::help::HelpKind {
                    #help.kind()
                }

                #[inline]
                fn name(&self) -> &str {
                    #help.name()
                }

                #[inline]
                fn alias(&self) -> Option<&str> {
                    #help.alias()
                }

                #[inline]
                fn description(&self) -> &str {
                    #help.description()
                }
            }
            __Help
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
                        if ty.mutability { quote! { &mut } } else { quote! { & } }
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
                crate::attr::is_subcommand(&path)
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

impl Eq for CommandAttrData{}

impl PartialEq for CommandAttrData {
    fn eq(&self, other: &Self) -> bool {
        self.fn_name == other.fn_name
    }
}

#[derive(Debug, Clone)]
pub struct FnArgData {
    pub arg_name: String,
    pub pat_type: PatType,
    pub attribute: Option<MacroAttribute>,
    pub name_value: Option<NameValueAttribute>,
    pub is_option: bool,
}

fn is_clapi_result_type(ty: &Type) -> bool {
    if ty.is_result() {
        return true;
    }

    matches!(ty.path().unwrap().as_str(), "clapi::Result" | "clapi::error::Result")
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
            !crate::attr::is_clapi_attribute(&path)
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
        let min = attribute.get(crate::attr::MIN)
            .map(|v| v.to_integer_literal::<usize>().expect("`min` must be a integer literal"))
            .unwrap_or(0);

        let max = attribute.get(crate::attr::MAX)
            .map(|v| v.to_integer_literal::<usize>().expect("`max` must be a integer literal"))
            .unwrap_or(1);

        let default = attribute.get(crate::attr::DEFAULT)
            .map(|v| v.to_bool_literal().expect("`default` must be a bool literal"))
            .unwrap_or(false);

        // Is an option bool flag is is boolean type and: min = 0, max = 1, default = false
        min == 0 && max == 1 && default == false
    } else {
        true
    }
}

mod imp {
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, Ordering};

    use syn::{AttrStyle, Attribute, AttributeArgs, FnArg, Item, ItemFn, PatType, Stmt, ItemStatic, File};

    use crate::arg::ArgAttrData;
    use crate::command::{drop_command_attributes, is_option_bool_flag, CommandAttrData, FnArgData};
    use crate::macro_attribute::{MacroAttribute, NameValueAttribute};
    use crate::option::OptionAttrData;
    use crate::utils::{path_to_string, NamePath};
    use crate::var::{ArgLocalVar, VarSource};
    use crate::attr;
    use crate::query::QueryItem;
    use syn::export::ToTokens;

    /// Implementation of `CommandAttrData::from_fn`
    pub fn command_from_fn(
        name_value_attr: NameValueAttribute,
        item_fn: ItemFn,
        is_child: bool,
        get_subcommands: bool,
        get_help: bool,
    ) -> CommandAttrData {
        let name = item_fn.sig.ident.to_string();
        command_from_fn_with_name(
            NamePath::new(name),
            name_value_attr,
            item_fn,
            is_child,
            get_subcommands,
            get_help,
        )
    }

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
                crate::attr::PARENT if is_child => {},
                crate::attr::DESCRIPTION => {
                    let description = value
                        .clone()
                        .to_string_literal()
                        .expect("`description` must be a string literal");
                    command.set_description(description);
                }
                crate::attr::ABOUT => {
                    let help = value
                        .clone()
                        .to_string_literal()
                        .expect("`about` must be a string literal");
                    command.set_description(help);
                }
                crate::attr::VERSION => {
                    assert!(
                        value.is_integer() || value.is_float() || value.is_string(),
                        "`version` must be an integer, float or string literal"
                    );
                    command.set_version(value.parse_literal::<String>().unwrap());
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
                let subcommand = command_from_fn(attribute.clone(), item_fn, true, true, false);
                subcommands.push((subcommand, attribute.clone()));
            }

            while let Some((subcommand, attribute)) = subcommands.pop() {
                if attribute.contains(crate::attr::PARENT) {
                    let literal = attribute.get(crate::attr::PARENT)
                        .unwrap()
                        .to_string_literal()
                        .expect("`parent` must be a string literal");

                    // If attribute was: #[subcommand(parent="")]
                    assert!(literal.trim().len() > 0, "`parent` was empty in `fn {}`", subcommand.fn_name.name());

                    let name_path = get_parent_path(literal, &subcommand.fn_name);

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
                            panic!("cannot find parent subcommand: `{}` for `{}`", name_path, subcommand.fn_name);
                        }
                    }
                } else {
                    command.set_child(subcommand);
                }
            }
        }

        // Get the `#[help]` static item if any
        if get_help {
            let mut result = item_fn.block.stmts.iter()
                .filter_map(|stmt| {
                    if let Stmt::Item(Item::Static(item_static)) = &stmt {
                        #[allow(clippy::blocks_in_if_conditions)]
                        if item_static.attrs.iter().any(|attribute| {
                            let macro_attr = MacroAttribute::new(attribute.clone());
                            assert!(macro_attr.is_empty(), "`#[help]` takes not params but was `{}`", macro_attr);
                            attr::is_help(macro_attr.path())
                        }) {
                            return Some(item_static.clone());
                        }
                    }

                    None
                }).collect::<Vec<ItemStatic>>();

            if result.len() > 1 {
                panic!("multiple `#[help]` defined");
            }

            if result.len() == 1 {
                command.set_help(result.remove(0));
            }
        }

        command.set_item_fn(drop_command_attributes(item_fn));
        command
    }

    fn get_subcommands_from_fn(item_fn: &ItemFn) -> Vec<(NameValueAttribute, ItemFn)> {
        let mut ret = Vec::new();

        for stmt in &item_fn.block.stmts {
            if let Stmt::Item(Item::Fn(item_fn)) = stmt {
                let subcommands = item_fn
                    .attrs
                    .iter()
                    .filter(|att| crate::attr::is_subcommand(path_to_string(&att.path).as_str()))
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

                    let name_value_attr =
                        if let Some(index) = inner_fn.attrs.iter().position(|att| {
                            crate::attr::is_subcommand(path_to_string(&att.path).as_str())
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

    fn get_fn_args(item_fn: &ItemFn) -> Vec<FnArgData> {
        fn get_fn_arg_ident_name(fn_arg: &FnArg) -> (String, PatType) {
            if let FnArg::Typed(pat_type) = &fn_arg {
                if let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                    return (pat_ident.ident.to_string(), pat_type.clone());
                }
            }

            panic!("`{}` is not a valid function arg", fn_arg.to_token_stream().to_string());
        }

        let mut ret = Vec::new();

        // We takes all the attributes that are `[arg(...)]` or `[option(...)]`
        let attributes = item_fn
            .attrs
            .iter()
            .cloned()
            .map(MacroAttribute::new)
            .filter(|attribute| crate::attr::is_option(attribute.path()) || crate::attr::is_arg(attribute.path()))
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
            if attributes[(index + 1)..].iter().any(|(arg, ..)| arg == name) {
                panic!("function argument `{}` is already used in `{}`", name, attribute);
            }
        }

        // Check the argument declared in the `option` or `arg` exists in the function
        for (path, _, _) in &attributes {
            if !fn_args.iter().any(|(arg_name, _)| arg_name == path) {
                panic!("argument `{}` is no defined in `fn {}`", path, item_fn.sig.ident);
            }
        }

        for (arg_name, pat_type) in fn_args {
            let (attribute, name_value) = if let Some(data) = attributes
                .iter()
                .find_map(|(path, attribute, name_value)| {
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
                .map(|attribute| crate::attr::is_option(attribute.path()))
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

    fn split_attr_path_and_name_values(attribute: MacroAttribute) -> (String, MacroAttribute, NameValueAttribute) {
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

    /// implementation of `CommandAttrData::from_path`
    pub fn command_from_path(
        args: AttributeArgs,
        item_fn: ItemFn,
        root_path: PathBuf,
    ) -> CommandAttrData {
        static IS_DEFINED: AtomicBool = AtomicBool::new(false);
        if let Ok(true) = IS_DEFINED.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed) {
            panic!("multiple `command` entry points defined: `{}`", item_fn.sig.ident);
        }

        let src = std::fs::read_to_string(&root_path).unwrap();
        let file = syn::parse_file(&src).unwrap();
        assert_is_top_free_function(&file, &item_fn);

        let attribute = NameValueAttribute::from_attribute_args(
            crate::attr::COMMAND, args, AttrStyle::Outer
        ).unwrap();

        let mut root = command_from_fn(attribute, item_fn, false, true, true);

        if let Some(help) = find_help(&root_path) {
            if root.help.is_some() {
                panic!("`#[help]` is already defined in `fn {}`", root.fn_name.name());
            }

            root.set_help(help)
        }

        let mut subcommands = get_subcommands_data(&root_path);

        while let Some((subcommand, _, attribute)) = subcommands.pop() {
            if attribute.contains(crate::attr::PARENT){
                let literal = attribute.get(crate::attr::PARENT)
                    .unwrap()
                    .to_string_literal()
                    .expect("`parent` must be a `string` literal");

                // If attribute was: #[subcommand(parent="")]
                assert!(literal.trim().len() > 0, "`parent` was empty in `fn {}`", subcommand.fn_name.name());

                let parent_name = get_parent_path(literal, &subcommand.fn_name);

                if parent_name == subcommand.fn_name {
                    panic!("self reference command parent in `{}`", subcommand.attribute);
                }

                if let Some(parent) = find_command_recursive(&mut root, &parent_name){
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
                        let parent_name = attribute.get(crate::attr::PARENT)
                            .unwrap()
                            .to_string_literal()
                            .unwrap();

                        panic!("cannot find parent command: `{}` for `{}`",
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

    fn find_help(root_path: &Path) -> Option<ItemStatic> {
        let mut result = crate::query::find_map_items(
            root_path, true, true, |item| {
            if let Item::Static(item_static) = item {
                let is_help = item_static.attrs.iter().any(|attribute| {
                    let macro_attr = MacroAttribute::new(attribute.clone());
                    assert!(macro_attr.is_empty(), "`#[help]` takes not params but was `{}`", macro_attr);
                    attr::is_help(macro_attr.path())
                });

                if is_help {
                    return Some(item_static.clone());
                }
            }
            None
        });

        if result.len() > 1 {
            panic!("multiple `#[help]` defined");
        }

        result.pop().map(|item| item.item)
    }

    fn find_command_recursive<'a>(command: &'a mut CommandAttrData, name: &'a NamePath) -> Option<&'a mut CommandAttrData> {
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

    fn get_parent_path(parent: String, current: &NamePath) -> NamePath {
        // `self` is not needed
        let parent_path = parent
            .split("::")
            .filter(|s| s != &"self")
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();

        // The path of the item ignore its name
        let mut path = Vec::from(current.item_path());

        // Navigate from the `current` to the `parent`
        for s in parent_path {
            if s == "super" {
                if path.pop().is_none() {
                    break
                }
            } else {
                path.push(s);
            }
        }

        if path.is_empty() {
            panic!("cannot find parent command `{}` for `{}`", parent, current.name());
        }

        NamePath::from_path(path)
    }

    fn get_subcommands_data(root_path: &Path) -> Vec<(CommandAttrData, PathBuf, NameValueAttribute)>{
        let query_data = get_subcommands_item_fn(root_path);
        let mut subcommands = Vec::new();

        for QueryItem {
            path,
            name_path,
            item: (item_fn, attr),
            ..
        } in query_data {
            let src = std::fs::read_to_string(&path).unwrap();
            let file = syn::parse_file(&src).unwrap();
            assert_is_top_free_function(&file, &item_fn);

            let command = command_from_fn_with_name(
                name_path,
                attr.clone(),
                item_fn,
                true,
                false,
                false
            );

            subcommands.push((command, path, attr));
        }

        subcommands
    }

    fn get_subcommands_item_fn(root_path: &Path) -> Vec<QueryItem<(ItemFn, NameValueAttribute)>> {
        fn if_subcommand_to_name_value(attribute: &Attribute) -> Option<NameValueAttribute>{
            if attr::is_subcommand(&path_to_string(&attribute.path)) {
                Some(MacroAttribute::new(attribute.clone())
                    .into_name_values()
                    .unwrap())
            } else {
                None
            }
        }

        crate::query::find_map_items(root_path, true, true, |item| {
            if let Item::Fn(item_fn) = item {
                if let Some(attribute) = item_fn.attrs
                    .iter()
                    .find_map(if_subcommand_to_name_value) {
                    return Some((item_fn.clone(), attribute));
                }
            }

            None
        })
    }

    fn assert_is_top_free_function(file: &File, item_fn: &ItemFn) {
        fn eq_item_fn(left: &ItemFn, right: &ItemFn) -> bool {
            left.block == right.block
                && left.sig == right.sig
                && left.vis == right.vis
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