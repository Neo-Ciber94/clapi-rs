use crate::args::ArgFromFn;
use crate::option::OptionFromFn;
use crate::utils::to_str_literal_stream2;
use crate::var::{ArgLocalVar, ArgType, LocalVarSource};
use crate::IteratorExt;
use clapi::utils::OptionExt;
use macro_attribute::{literal_to_string, MacroAttribute, NameValueAttribute, Value};
use proc_macro2::TokenStream;
use quote::*;
use syn::export::ToTokens;
use syn::{Attribute, AttributeArgs, FnArg, Item, ItemFn, PatType, Stmt};

/// Tokens for:
///
/// ```ignore
/// #[command(
///     description="A description",
///     help="Help text",
/// )]
/// ```
#[derive(Debug)]
pub struct CommandFromFn {
    name: String,
    is_child: bool,
    description: Option<String>,
    help: Option<String>,
    item_fn: Option<ItemFn>,
    children: Vec<CommandFromFn>,
    options: Vec<OptionFromFn>,
    arg: Option<ArgFromFn>,
    vars: Vec<ArgLocalVar>,
}

impl CommandFromFn {
    fn new(name: String) -> Self {
        CommandFromFn {
            name,
            description: None,
            help: None,
            item_fn: None,
            children: vec![],
            options: vec![],
            vars: vec![],
            arg: None,
            is_child: false,
        }
    }

    fn new_child(name: String) -> Self {
        CommandFromFn {
            name,
            description: None,
            help: None,
            item_fn: None,
            children: vec![],
            options: vec![],
            vars: vec![],
            arg: None,
            is_child: true,
        }
    }

    pub fn from_fn(args: AttributeArgs, func: ItemFn) -> Self {
        let name = func.sig.ident.to_string();
        let attr_data = NameValueAttribute::from_attribute_args(name.as_str(), args).unwrap();
        new_command_from_fn(attr_data, func, false, true)
    }

    pub fn from_file(args: AttributeArgs, func: ItemFn, file: syn::File) -> Self {
        command_file::new_command_from_file(args, func, file)
    }

    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
    }

    pub fn set_help(&mut self, help: String) {
        self.help = Some(help);
    }

    pub fn set_child(&mut self, command: CommandFromFn) {
        self.children.push(command)
    }

    pub fn set_option(&mut self, option: OptionFromFn) {
        self.options.push(option);
    }

    pub fn set_args(&mut self, args: ArgFromFn) {
        self.arg = Some(args);
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
            "ItemFn is not set for command `{}`",
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
            .arg
            .as_ref()
            .map(|tokens| quote! { .set_args(#tokens)})
            .unwrap_or_else(|| quote! {});

        // Command description
        let description = self
            .description
            .as_ref()
            .map(|s| to_str_literal_stream2(s).unwrap())
            .map(|tokens| quote! { .set_description(#tokens)})
            .unwrap_or_else(|| quote! {});

        // Command help
        let help = self
            .help
            .as_ref()
            .map(|s| to_str_literal_stream2(s).unwrap())
            .map(|tokens| quote! { .set_help(#tokens)})
            .unwrap_or_else(|| quote! {});

        // Get the function body
        let body = self.get_body(vars.as_slice());

        // Command or RootCommand `new`
        let mut command = if self.is_child {
            let name_str = to_str_literal_stream2(&self.name).unwrap();
            quote! { clapi::command::Command::new(#name_str) }
        } else {
            quote! { clapi::root_command::RootCommand::new() }
        };

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
            let attrs = &self.item_fn.as_ref().unwrap().attrs;
            let outer = self.outer_body();
            //let ret = &self.item_fn.as_ref().unwrap().sig.output;
            quote! {
                #(#attrs)*
                fn #name() {
                    #(#outer)*

                    let command = #command ;
                    clapi::command_line::CommandLine::new(command)
                        .use_default_help()
                        .use_default_suggestions()
                        .run()
                        .expect("an error occurred");
                }
            }
        }
    }

    fn get_body(&self, vars: &[TokenStream]) -> TokenStream {
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
                #fn_name(#(#inputs,)*);
                Ok(())
            }
        } else {
            let statements = self.inner_body();

            quote! {
                #(#vars)*
                #(#statements)*
                Ok(())
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
        // functions, struct, const, statics, ...
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

impl ToTokens for CommandFromFn {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.expand().into_iter())
    }
}

// pub struct AttrKeys;
// impl AttrKeys {
//     const OPTION: &'static str = "option";
//     const ARG: &'static str = "arg";
//     const SUBCOMMAND: &'static str = "subcommand";
//     const DESCRIPTION: &'static str = "description";
//     const ALIAS: &'static str = "alias";
//     const MIN: &'static str = "min";
//     const MAX: &'static str = "max";
//     const DEFAULT: &'static str = "default";
// }

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

fn new_command_from_fn(
    name_value_attr: NameValueAttribute,
    item_fn: ItemFn,
    is_child: bool,
    get_subcommands: bool,
) -> CommandFromFn {
    let name = item_fn.sig.ident.to_string();
    let mut command = if is_child {
        CommandFromFn::new_child(name)
    } else {
        CommandFromFn::new(name)
    };

    for (key, value) in &name_value_attr {
        match key.as_str() {
            "description" => {
                let description = value
                    .clone()
                    .as_string_literal()
                    .expect("`description` is expected to be string literal");
                command.set_description(description);
            }
            "help" => {
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
        .filter(|att| att.path() == "option" || att.path() == "arg")
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
    let mut arg_count = 0;

    // Pass function arguments in order
    for arg in &fn_args {
        if arg.is_option {
            command.set_var(ArgLocalVar::new(arg.pat_type.clone(), LocalVarSource::Opts));
        } else {
            command.set_var(ArgLocalVar::new(
                arg.pat_type.clone(),
                LocalVarSource::Args(arg_count),
            ));

            arg_count += 1;
        }
    }

    // Add args
    let arg_count = fn_args.iter().filter(|f| !f.is_option).count();
    if arg_count > 0 {
        if let Some(fn_arg) = fn_args.iter().filter(|f| !f.is_option).single() {
            let command_args =
                ArgFromFn::from_attribute_data(fn_arg.attr.clone().unwrap(), &fn_arg.pat_type);
            command.set_args(command_args);
        } else {
            unimplemented!("multiple args is not supported");
            // let mut command_args = ArgAttribute::new(None);
            // command_args.set_max(arg_count);
            // command.set_args(command_args);
        }
    }

    // Add options
    for arg in fn_args.iter().filter(|n| n.is_option) {
        let mut option = OptionFromFn::new(arg.arg_name.clone());
        let mut args = ArgFromFn::new(&arg.pat_type);

        if let Some(att) = &arg.attr {
            for (key, value) in att {
                match key.as_str() {
                    "name" => { /* Ignore */ }
                    "alias" => {
                        let alias = value
                            .clone()
                            .as_string_literal()
                            .expect("option `alias` is expected to be string literal");

                        option.set_alias(alias);
                    }
                    "description" => {
                        let description = value
                            .clone()
                            .as_string_literal()
                            .expect("option `description` is expected to be string literal");
                        option.set_description(description);
                    }
                    "min" => {
                        let min = value
                            .clone()
                            .parse_literal::<usize>()
                            .expect("option `min` is expected to be an integer literal");

                        args.set_min(min);
                    }
                    "max" => {
                        let max = value
                            .clone()
                            .parse_literal::<usize>()
                            .expect("option `max` is expected to be an integer literal");

                        args.set_max(max);
                    }
                    "default" => match value {
                        Value::Literal(lit) => args.set_default_values(vec![lit.clone()]),
                        Value::Array(array) => args.set_default_values(array.clone()),
                    },
                    _ => panic!("invalid {} key `{}`", att.path(), key),
                }
            }
        }

        option.set_args(args);
        command.set_option(option);
    }

    // Add children
    if get_subcommands {
        for (name_value, item_fn) in get_subcommands_from_fn(&item_fn) {
            command.set_child(new_command_from_fn(name_value, item_fn, true, true));
        }
    }

    command.set_item_fn(drop_command_attributes(item_fn));
    // command.set_item_fn(item_fn);
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
                    .filter(|att| att.path.to_token_stream().to_string() == "subcommand")
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

                    let attr = if let Some(index) = inner_fn
                        .attrs
                        .iter()
                        .position(|att| att.path.to_token_stream().to_string() == "subcommand")
                    {
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
                    att.get("name")
                        .map(|v| v.as_string_literal())
                        .flatten()
                        .contains_some(&fn_arg.name)
                })
                .cloned();

            let arg_name = fn_arg.name.clone();
            let is_option = match &attr {
                Some(att) => att.path() == "option",
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

pub fn drop_command_attributes(mut item_fn: ItemFn) -> ItemFn {
    item_fn.attrs = item_fn
        .attrs
        .iter()
        .filter(|att| {
            let path = att.path.to_token_stream().to_string();
            match path.as_str() {
                "command" | "subcommand" | "option" | "arg" => false,
                _ => true,
            }
        })
        .cloned()
        .collect::<Vec<Attribute>>();

    item_fn
}

fn assert_attr_name_match_fn_arg(
    item_fn: &ItemFn,
    fn_args: &[NamedFnArg],
    attr: &[NameValueAttribute],
) {
    for name_value in attr {
        if let Some(Value::Literal(lit)) = name_value.get("name") {
            let name = literal_to_string(lit);
            let contains_name = fn_args.iter().any(|arg| arg.name == name);
            assert!(
                contains_name,
                "cannot find function argument named `{}` in function `{}`",
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
        if let Some(value) = att.get("name") {
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

#[allow(unused_variables)]
mod command_file {
    use crate::command::{new_command_from_fn, CommandFromFn};
    use macro_attribute::NameValueAttribute;
    use std::convert::TryFrom;
    use syn::export::ToTokens;
    use syn::{AttributeArgs, File, Item, ItemFn};

    pub fn new_command_from_file(
        args: AttributeArgs,
        item_fn: ItemFn,
        file: File,
    ) -> CommandFromFn {
        assert_one_root_command(&item_fn.sig.ident.to_string(), &file);

        let attr = NameValueAttribute::from_attribute_args("command", args).unwrap();
        let mut command = new_command_from_fn(attr, item_fn, false, false);
        let mut subcommands = get_subcommands_from_file(&file);

        // Push all the registered commands
        subcommands.extend(get_registered_subcommands());

        for (att, item_fn) in subcommands {
            let subcommand = new_command_from_fn(att.clone(), item_fn.clone(), true, true);
            command.set_child(subcommand);
        }

        command
    }

    fn get_registered_subcommands() -> Vec<(NameValueAttribute, ItemFn)> {
        let mut ret = Vec::new();

        for data in crate::shared::get_subcommand_registry().iter() {
            let args = data.parse_args().expect("invalid attribute");
            let item_fn = data
                .parse_item_fn()
                .expect("invalid item for attribute, expected function");
            let attr = NameValueAttribute::from_attribute_args("subcommand", args).unwrap();
            ret.push((attr, item_fn));
        }
        ret
    }

    fn get_subcommands_from_file(file: &File) -> Vec<(NameValueAttribute, ItemFn)> {
        let mut ret = Vec::new();

        for item in &file.items {
            if let Item::Fn(item_fn) = item {
                if let Some(att) = item_fn
                    .attrs
                    .iter()
                    .find(|a| a.path.to_token_stream().to_string() == "subcommand")
                {
                    let attr = NameValueAttribute::try_from(att.clone()).unwrap();
                    ret.push((attr, item_fn.clone()))
                }
            }
        }

        ret
    }

    fn assert_one_root_command(root_fn_name: &str, file: &File) {
        let fns = file
            .items
            .iter()
            .filter_map(|x| match x {
                Item::Fn(item_fn) => Some(item_fn),
                _ => None,
            })
            .filter(|x| x.attrs.len() > 0)
            .filter(|x| x.sig.ident.to_string() != root_fn_name);

        for f in fns {
            let command_attr = f
                .attrs
                .iter()
                .find_map(|att| att.path.segments.last())
                .filter(|att| att.ident.to_string() == "command");

            assert!(
                command_attr.is_none(),
                "can only exists 1 command attribute per file"
            );
        }
    }
}
