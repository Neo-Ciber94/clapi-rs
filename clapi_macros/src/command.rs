use crate::args::ArgData;
use crate::keys;
use crate::option::OptionData;
use crate::var::{ArgLocalVar, ArgType};
use macro_attribute::NameValueAttribute;
use proc_macro2::TokenStream;
use quote::*;
use syn::export::ToTokens;
use syn::{Attribute, AttributeArgs, FnArg, ItemFn, PatType, ReturnType, Stmt};

use crate::TypeExtensions;
use command_file::new_command_from_file;
use command_fn::new_command;
use std::path::PathBuf;

/// Tokens for:
///
/// ```text
/// #[command(
///     description="A description",
///     help="Help text",
/// )]
/// ```
#[derive(Debug)]
pub struct CommandData {
    name: String,
    is_child: bool,
    description: Option<String>,
    help: Option<String>,
    item_fn: Option<ItemFn>,
    children: Vec<CommandData>,
    options: Vec<OptionData>,
    args: Option<ArgData>,
    vars: Vec<ArgLocalVar>,
}

impl CommandData {
    fn new(name: String, is_child: bool) -> Self {
        CommandData {
            name,
            is_child,
            description: None,
            help: None,
            item_fn: None,
            children: vec![],
            options: vec![],
            vars: vec![],
            args: None,
        }
    }

    pub fn from_fn(args: AttributeArgs, func: ItemFn) -> Self {
        let name = func.sig.ident.to_string();
        let attr_data = NameValueAttribute::from_attribute_args(name.as_str(), args).unwrap();
        new_command(attr_data, func, false, true)
    }

    pub fn from_file(args: AttributeArgs, func: ItemFn, path: PathBuf, file: syn::File) -> Self {
        new_command_from_file(args, func, path, file)
    }

    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
    }

    pub fn set_help(&mut self, help: String) {
        self.help = Some(help);
    }

    pub fn set_child(&mut self, command: CommandData) {
        assert!(command.is_child);
        self.children.push(command)
    }

    pub fn set_option(&mut self, option: OptionData) {
        self.options.push(option);
    }

    pub fn set_args(&mut self, args: ArgData) {
        self.args = Some(args);
    }

    pub fn set_var(&mut self, var: ArgLocalVar) {
        self.vars.push(var);
    }

    pub fn set_item_fn(&mut self, item_fn: ItemFn) {
        self.item_fn = Some(item_fn);
    }

    pub fn expand(&self) -> TokenStream {
        assert!(
            self.item_fn.is_some(),
            "`ItemFn` is not set for command `{}`",
            self.name
        );

        // Command options
        let options = self
            .options
            .iter()
            .map(|x| quote! { .set_option(#x)})
            .collect::<Vec<TokenStream>>();

        // Command children
        let children = self
            .children
            .iter()
            .map(|x| quote! { .set_command(#x)})
            .collect::<Vec<TokenStream>>();

        // Command function variables
        let vars = self
            .vars
            .iter()
            .map(|x| quote! { #x })
            .collect::<Vec<TokenStream>>();

        // Command args
        let args = self
            .args
            .as_ref()
            .map(|tokens| quote! { .set_args(#tokens)})
            .unwrap_or_else(|| quote! {});

        // Command description
        let description = self
            .description
            .as_ref()
            .map(|s| quote! { #s })
            .map(|tokens| quote! { .set_description(#tokens)})
            .unwrap_or_else(|| quote! {});

        // Command help
        let help = self
            .help
            .as_ref()
            .map(|s| quote! { #s })
            .map(|tokens| quote! { .set_help(#tokens)})
            .unwrap_or_else(|| quote! {});

        // Get the function body
        let body = self.get_body(vars.as_slice());

        // Instantiate `Command` or `RootCommand`
        let mut command = if self.is_child {
            let command_name = quote_expr!(self.name);
            quote! { clapi::command::Command::new(#command_name) }
        } else {
            quote! { clapi::root_command::RootCommand::new() }
        };

        // Build the command
        command = quote! {
            #command
                #description
                #help
                #args
                #(#options)*
                #(#children)*
                .set_handler(|opts, args|{
                    #body
                })
        };

        if self.is_child {
            command
        } else {
            let name = self.name.as_str().parse::<TokenStream>().unwrap();
            let ret = &self.item_fn.as_ref().unwrap().sig.output;
            let attrs = &self.item_fn.as_ref().unwrap().attrs;
            let outer = self.outer_body();
            let error_handling = match ret {
                ReturnType::Type(_, ty) if ty.is_result() => quote! {},
                _ => quote! { .expect("an error occurred"); },
            };

            // Emit the tokens to create the function with the `RootCommand`
            quote! {
                #(#attrs)*
                fn #name() #ret {
                    #(#outer)*

                    let command = #command ;
                    clapi::command_line::CommandLine::new(command)
                        .use_default_help()
                        .use_default_suggestions()
                        .run()
                        #error_handling
                }
            }
        }
    }

    fn get_body(&self, vars: &[TokenStream]) -> TokenStream {
        let ret = &self.item_fn.as_ref().unwrap().sig.output;
        let error_handling = match ret {
            ReturnType::Type(_, ty) if ty.is_result() => quote! {},
            // If function is not `Result` we need return `fn_name(args) ; Ok(())`
            _ => quote! { ; Ok(()) },
        };

        if self.is_child {
            let fn_name = self.name.parse::<TokenStream>().unwrap();
            let inputs = self.vars.iter().map(|var| {
                let var_name = var.name().parse::<TokenStream>().unwrap();
                let is_ref = match var.arg_type() {
                    ArgType::Slice(_) => quote! { & },
                    ArgType::MutSlice(_) => quote! { &mut },
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
            let statements = self.inner_body();

            quote! {
                #(#vars)*
                #(#statements)*
                #error_handling
            }
        }
    }

    fn inner_body(&self) -> Vec<TokenStream> {
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

    fn outer_body(&self) -> Vec<TokenStream> {
        // functions, struct, impl, const, statics, ...
        self.item_fn
            .as_ref()
            .unwrap()
            .block
            .stmts
            .iter()
            .filter(|s| matches!(s, Stmt::Item(_)))
            .map(|s| s.to_token_stream())
            .collect()
    }
}

impl ToTokens for CommandData {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.expand().into_iter())
    }
}

#[derive(Debug)]
struct CommandFnArg {
    arg_name: String,
    pat_type: PatType,
    attr: Option<NameValueAttribute>,
    is_option: bool,
}

#[derive(Debug)]
struct NamedFnArg {
    name: String,
    fn_arg: FnArg,
}

pub fn drop_command_attributes(mut item_fn: ItemFn) -> ItemFn {
    item_fn.attrs = item_fn
        .attrs
        .iter()
        .filter(|att| {
            let path = att.path.to_token_stream().to_string();
            !keys::is_clapi_attribute(&path)
        })
        .cloned()
        .collect::<Vec<Attribute>>();

    item_fn
}

mod command_fn {
    use clapi::utils::OptionExt;
    use macro_attribute::{literal_to_string, MacroAttribute, NameValueAttribute, Value};
    use syn::export::ToTokens;
    use syn::{Attribute, FnArg, Item, ItemFn, PatType, Stmt, Type};

    use crate::args::ArgData;
    use crate::command::{drop_command_attributes, CommandData, CommandFnArg, NamedFnArg};
    use crate::keys;
    use crate::option::OptionData;
    use crate::utils::pat_type_to_string;
    use crate::var::{ArgLocalVar, VarSource};
    use crate::{IteratorExt, TypeExtensions};

    pub fn new_command(
        name_value_attr: NameValueAttribute,
        item_fn: ItemFn,
        is_child: bool,
        get_subcommands: bool,
    ) -> CommandData {
        let name = item_fn.sig.ident.to_string();
        new_command_with_name(name_value_attr, item_fn, name, is_child, get_subcommands)
    }

    pub fn new_command_with_name(
        name_value_attr: NameValueAttribute,
        item_fn: ItemFn,
        name: String,
        is_child: bool,
        get_subcommands: bool,
    ) -> CommandData {
        let mut command = CommandData::new(name, is_child);

        for (key, value) in &name_value_attr {
            match key.as_str() {
                keys::DESCRIPTION => {
                    let description = value
                        .clone()
                        .as_string_literal()
                        .expect("`description` is expected to be string literal");
                    command.set_description(description);
                }
                keys::HELP => {
                    let help = value
                        .clone()
                        .as_string_literal()
                        .expect("`help` is expected to be string literal");
                    command.set_description(help);
                }
                _ => panic!("invalid {} key `{}`", name_value_attr.path(), key),
            }
        }

        let attrs = item_fn
            .attrs
            .iter()
            .cloned()
            .map(|att| MacroAttribute::new(att).into_name_values().unwrap())
            .filter(|att| att.path() == keys::OPTION || att.path() == keys::ARG)
            .collect::<Vec<NameValueAttribute>>();

        let named_fn_args = item_fn
            .sig
            .inputs
            .iter()
            .cloned()
            .map(|fn_arg| {
                if let FnArg::Typed(pat_type) = &fn_arg {
                    let name = pat_type.pat.to_token_stream().to_string();
                    let fn_arg = fn_arg.clone();
                    NamedFnArg { name, fn_arg }
                } else {
                    panic!("FnArg is not a free function")
                }
            })
            .collect::<Vec<NamedFnArg>>();

        // Check all attributes have the `name` key declared, is a literal and is not empty
        assert_attributes_name_is_declared(&attrs);

        // Check all attributes `name` match a function arg
        assert_attr_name_match_fn_arg(&item_fn, &named_fn_args, &attrs);

        let fn_args = get_fn_args(&attrs, &named_fn_args);
        let arg_count = fn_args.iter().filter(|f| !f.is_option).count();
        let mut arg_index = 0;

        // Pass function arguments in order
        for fn_arg in &fn_args {
            if fn_arg.is_option {
                let source = if is_implicit_bool_arg(fn_arg) { VarSource::OptBool } else { VarSource::Opts };
                command.set_var(ArgLocalVar::new(fn_arg.pat_type.clone(), source));
            } else {
                if arg_count > 1 {
                    let ty = fn_arg.pat_type.ty.as_ref();
                    if ty.is_slice() || ty.is_vec() {
                        panic!("invalid argument type for: `{}`\
                        \nwhen multiples `arg` are defined, arguments cannot be declared as `Vec` or `slice`",
                          pat_type_to_string(&fn_arg.pat_type));
                    }

                    if let Some(attr) = &fn_arg.attr {
                        assert!(
                            attr.get("default").is_none(),
                            "`default` is not supported when multiple arguments are defined"
                        );
                        assert!(
                            attr.get("min").is_none(),
                            "`min` is not supported when multiple arguments are defined"
                        );
                        assert!(
                            attr.get("max").is_none(),
                            "`min` is not supported when multiple arguments are defined"
                        );
                    }
                }

                command.set_var(ArgLocalVar::new(
                    fn_arg.pat_type.clone(),
                    VarSource::Args(arg_index),
                ));

                arg_index += 1;
            }
        }

        // Add args
        if arg_count > 0 {
            if let Some(fn_arg) = fn_args.iter().filter(|f| !f.is_option).single() {
                command.set_args(ArgData::from_attribute(
                    fn_arg.attr.clone().unwrap(),
                    &fn_arg.pat_type,
                ));
            } else {
                // unimplemented!("multiple args is not supported");
                let mut args = ArgData::new();
                args.set_min(arg_count);
                args.set_max(arg_count);
                command.set_args(args);
            }
        }

        // Add options
        for fn_arg in fn_args.iter().filter(|n| n.is_option) {
            let mut option = OptionData::new(fn_arg.arg_name.clone());
            let mut args = ArgData::from_pat_type(&fn_arg.pat_type);

            // Arguments that belong to options don't will be named
            args.set_name(None);

            if let Some(att) = &fn_arg.attr {
                for (key, value) in att {
                    match key.as_str() {
                        keys::NAME => { /* Ignore */ }
                        keys::ALIAS => {
                            let alias = value
                                .clone()
                                .as_string_literal()
                                .expect("option `alias` is expected to be string literal");

                            option.set_alias(alias);
                        }
                        keys::DESCRIPTION => {
                            let description = value
                                .clone()
                                .as_string_literal()
                                .expect("option `description` is expected to be string literal");
                            option.set_description(description);
                        }
                        keys::MIN => {
                            let min = value
                                .clone()
                                .parse_literal::<usize>()
                                .expect("option `min` is expected to be an integer literal");

                            args.set_min(min);
                        }
                        keys::MAX => {
                            let max = value
                                .clone()
                                .parse_literal::<usize>()
                                .expect("option `max` is expected to be an integer literal");

                            args.set_max(max);
                        }
                        keys::DEFAULT => match value {
                            Value::Literal(lit) => args.set_default_values(vec![lit.clone()]),
                            Value::Array(array) => args.set_default_values(array.clone()),
                        },
                        _ => panic!("invalid {} key `{}`", att.path(), key),
                    }
                }
            }

            // An argument is considered implicit if:
            // - Is bool type
            // - Don't contains `min`, `max` or `default`
            if !is_implicit_bool_arg(fn_arg) {
                option.set_args(args);
            }

            command.set_option(option);
        }

        // Add children
        if get_subcommands {
            for (name_value, item_fn) in get_subcommands_from_fn(&item_fn) {
                command.set_child(new_command(name_value, item_fn, true, true));
            }
        }

        command.set_item_fn(drop_command_attributes(item_fn));
        command
    }

    fn get_subcommands_from_fn(item_fn: &ItemFn) -> Vec<(NameValueAttribute, ItemFn)> {
        let mut ret = Vec::new();

        for stmt in &item_fn.block.stmts {
            if let Stmt::Item(item) = stmt {
                if let Item::Fn(item_fn) = item {
                    let subcommands = item_fn
                        .attrs
                        .iter()
                        .filter(|att| att.path.to_token_stream().to_string() == keys::SUBCOMMAND)
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

                        let attr = if let Some(index) = inner_fn.attrs.iter().position(|att| {
                            att.path.to_token_stream().to_string() == keys::SUBCOMMAND
                        }) {
                            MacroAttribute::new(inner_fn.attrs.swap_remove(index))
                                .into_name_values()
                                .unwrap()
                        } else {
                            unreachable!()
                        };

                        ret.push((attr, inner_fn))
                    }
                }
            }
        }

        ret
    }

    fn get_fn_args(attrs: &[NameValueAttribute], fn_args: &[NamedFnArg]) -> Vec<CommandFnArg> {
        let mut ret = Vec::new();

        for fn_arg in fn_args {
            if let FnArg::Typed(pat_type) = fn_arg.fn_arg.clone() {
                let attr = attrs
                    .iter()
                    .find(|att| {
                        att.get(keys::NAME)
                            .map(|v| v.as_string_literal())
                            .flatten()
                            .contains_some(&fn_arg.name)
                    })
                    .cloned();

                let arg_name = fn_arg.name.clone();
                let is_option = match &attr {
                    Some(att) => att.path() == keys::OPTION,
                    None => true,
                };

                ret.push(CommandFnArg {
                    arg_name,
                    pat_type,
                    attr,
                    is_option,
                });
            }
        }

        ret
    }

    fn is_implicit_bool_arg(fn_arg: &CommandFnArg) -> bool {
        if let Some(attr) = &fn_arg.attr {
            fn_arg.pat_type.ty.is_bool()
                && !(attr.contains_name("min")
                    || attr.contains_name("max")
                    || attr.contains_name("default"))
        } else {
            fn_arg.pat_type.ty.is_bool()
        }
    }

    fn assert_is_non_vec_or_slice(ty: &Type, pat_type: &PatType) {
        if !ty.is_vec() || !ty.is_slice() {
            panic!(
                "invalid argument type for: `{}`\
                \nwhen multiples `arg` are defined, arguments cannot be declared as `Vec` or `slice`",
                pat_type_to_string(pat_type)
            );
        }
    }

    fn assert_attr_name_match_fn_arg(
        item_fn: &ItemFn,
        fn_args: &[NamedFnArg],
        attr: &[NameValueAttribute],
    ) {
        for name_value in attr {
            if let Some(Value::Literal(lit)) = name_value.get(keys::NAME) {
                let name = literal_to_string(lit);
                let contains_name = fn_args.iter().any(|arg| arg.name == name);

                assert!(
                    contains_name,
                    "cannot find function argument named `{}` in `{}` function",
                    name,
                    item_fn.sig.ident.to_string()
                )
            } else {
                panic!("expected string literal for `name`")
            }
        }
    }

    fn assert_attributes_name_is_declared(fn_attrs: &[NameValueAttribute]) {
        for att in fn_attrs {
            if let Some(value) = att.get(keys::NAME) {
                assert!(value.is_string(), "`name` must be a string literal");
                assert!(
                    !value.as_string_literal().unwrap().is_empty(),
                    "`name` cannot be empty"
                );
            } else {
                panic!("`name` is required in `{}`", att.path());
            }
        }
    }
}

mod command_file {
    use crate::command::{new_command, CommandData};
    use crate::keys;
    use crate::utils::path_to_string;
    use macro_attribute::NameValueAttribute;
    use std::convert::TryFrom;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, Ordering};
    use syn::export::ToTokens;
    use syn::{AttributeArgs, File, Item, ItemFn, ItemMod};

    pub struct CommandFromPath {
        path: PathBuf,
        attr: NameValueAttribute,
        item_fn: ItemFn,
    }

    pub fn new_command_from_file(
        args: AttributeArgs,
        item_fn: ItemFn,
        path: PathBuf,
        file: File,
    ) -> CommandData {
        static IS_DEFINED: AtomicBool = AtomicBool::new(false);

        if IS_DEFINED.load(Ordering::Relaxed) {
            panic!(
                "multiple `command` entry points defined: `{}`",
                item_fn.sig.ident
            );
        } else {
            IS_DEFINED.store(true, Ordering::Relaxed);
        }

        let attr = NameValueAttribute::from_attribute_args(keys::COMMAND, args).unwrap();
        let mut command = new_command(attr, item_fn.clone(), false, true);

        for (att, item_fn) in find_subcommands(path.as_path(), &file) {
            let subcommand = new_command(att.clone(), item_fn.clone(), true, true);
            command.set_child(subcommand);
        }

        command
    }

    fn find_subcommands(path: &Path, file: &File) -> Vec<(NameValueAttribute, ItemFn)> {
        let mut subcommands = get_subcommands_from_file(file);
        subcommands.extend(get_registered_subcommands(path));
        subcommands
    }

    fn get_subcommand_attribute(item_fn: &ItemFn) -> Option<NameValueAttribute> {
        if let Some(att) = item_fn
            .attrs
            .iter()
            .find(|a| a.path.to_token_stream().to_string() == keys::SUBCOMMAND)
        {
            Some(NameValueAttribute::try_from(att.clone()).unwrap())
        } else {
            None
        }
    }

    fn get_registered_subcommands(path: &Path) -> Vec<(NameValueAttribute, ItemFn)> {
        let mut ret = Vec::new();
        for data in crate::shared::get_subcommand_registry().iter() {
            let args = data.parse_args().expect("invalid attribute");
            let item_fn = data
                .parse_item_fn()
                .expect("invalid item for attribute, expected function");

            if path != data.path() {
                panic!(
                    "`{}` subcommand must be defined in the same file than the root `command`.\
                \nbut was defined in `{}`",
                    item_fn.sig.ident,
                    data.path().display()
                );
            }

            let attr = NameValueAttribute::from_attribute_args(keys::SUBCOMMAND, args).unwrap();
            ret.push((attr, item_fn));
        }
        ret
    }

    fn get_subcommands_from_file(file: &File) -> Vec<(NameValueAttribute, ItemFn)> {
        let mut ret = Vec::new();
        for item in &file.items {
            match item {
                Item::Fn(item_fn) => {
                    if let Some(attr) = get_subcommand_attribute(item_fn) {
                        ret.push((attr, item_fn.clone()));
                    }
                }
                _ => {}
            }
        }

        ret
    }

    fn get_subcommands_from_module(module: &ItemMod) -> Vec<(NameValueAttribute, ItemFn)> {
        let mut ret = Vec::new();

        if let Some((_, content)) = module.content.as_ref() {
            for item in content {
                if let Item::Fn(item_fn) = item {
                    let attr = get_subcommand_attribute(item_fn).unwrap();
                    ret.push((attr, item_fn.clone()));
                }
            }
        }

        ret
    }

    fn get_subcommands_from_path(path: &Path) -> Vec<CommandFromPath> {
        let content = std::fs::read_to_string(path).unwrap();
        let file = syn::parse_file(content.as_str()).unwrap();
        let mut subcommands = Vec::new();

        for item in file.items {
            if let Some(item_fn) = matches_map!(item, Item::Fn(f) => f) {
                let name_value_attr = item_fn
                    .attrs
                    .iter()
                    .find(|att| keys::is_subcommand(&path_to_string(&att.path)))
                    .map(|att| NameValueAttribute::try_from(att.clone()).unwrap());

                if let Some(attr) = name_value_attr {
                    subcommands.push(CommandFromPath {
                        path: path.to_path_buf(),
                        attr,
                        item_fn,
                    })
                }
            }
        }

        subcommands
    }
}
