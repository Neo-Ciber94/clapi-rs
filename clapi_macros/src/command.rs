use crate::args::ArgAttribute;
use crate::attr_data::{literal_to_string, AttributeData, Value};
use crate::option::OptionAttribute;
use crate::var::{ArgLocalVar, ArgType, LocalVarSource};
use clapi::utils::OptionExt;
use proc_macro2::TokenStream;
use quote::*;
use syn::export::ToTokens;
use syn::{Attribute, AttributeArgs, Block, FnArg, Item, ItemFn, Pat, PatType, Stmt};
use crate::IteratorExt;
use crate::parser::parse_to_str_stream2;

/// Tokens for:
///
/// ```ignore
/// #[command(
///     description="A description",
///     help="Help text",
/// )]
/// ```
#[derive(Debug)]
pub struct CommandAttribute {
    name: String,
    description: Option<String>,
    help: Option<String>,
    item_fn: Option<ItemFn>,
    children: Vec<CommandAttribute>,
    options: Vec<OptionAttribute>,
    args: Option<ArgAttribute>,
    vars: Vec<ArgLocalVar>,
    is_child: bool,
}

impl CommandAttribute {
    pub(crate) fn new(name: String) -> Self {
        CommandAttribute {
            name,
            description: None,
            help: None,
            item_fn: None,
            children: vec![],
            options: vec![],
            vars: vec![],
            args: None,
            is_child: false,
        }
    }

    pub fn new_child(name: String) -> Self {
        CommandAttribute {
            name,
            description: None,
            help: None,
            item_fn: None,
            children: vec![],
            options: vec![],
            vars: vec![],
            args: None,
            is_child: true,
        }
    }

    pub fn from_attribute_args(args: AttributeArgs, func: ItemFn) -> Self {
        let name = func.sig.ident.to_string();
        let attr_data = AttributeData::from_attribute_args(name, args);
        new_command_tokens(attr_data, func, false)
    }

    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
    }

    pub fn set_help(&mut self, help: String) {
        self.help = Some(help);
    }

    pub fn set_child(&mut self, command: CommandAttribute) {
        self.children.push(command)
    }

    pub fn set_option(&mut self, option: OptionAttribute) {
        self.options.push(option);
    }

    pub fn set_args(&mut self, args: ArgAttribute) {
        self.args = Some(args);
    }

    pub fn set_var(&mut self, var: ArgLocalVar) {
        self.vars.push(var);
    }

    pub fn set_fn(&mut self, item_fn: ItemFn) {
        self.item_fn = Some(item_fn);
    }

    pub fn expand(&self) -> TokenStream {
        let mut options = Vec::new();
        let mut children = Vec::new();
        let mut vars = Vec::new();

        for option in &self.options {
            options.push(quote! { .set_option(#option) });
        }

        for child in &self.children {
            children.push(quote! { .set_command(#child) });
        }

        for var in &self.vars {
            vars.push(quote! { #var });
        }

        let args = self
            .args
            .as_ref()
            .map(|tokens| quote! { .set_args(#tokens)})
            .unwrap_or_else(|| quote! {});

        let description = self
            .description
            .as_ref()
            .map(|s| parse_to_str_stream2(s).unwrap())
            .map(|tokens| quote! { .set_description(#tokens)})
            .unwrap_or_else(|| quote! {});

        let help = self
            .help
            .as_ref()
            .map(|s| parse_to_str_stream2(s).unwrap())
            .map(|tokens| quote! { .set_help(#tokens)})
            .unwrap_or_else(|| quote! {});

        // Get the function body
        let body = self.get_body(vars.as_slice());

        let command = if self.is_child {
            let name_str = parse_to_str_stream2(&self.name).unwrap();
            quote! { clapi::command::Command::new(#name_str) }
        } else {
            quote! { clapi::root_command::RootCommand::new() }
        };

        let mut inner = quote! {
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

        if !self.is_child {
            inner = quote! {
                let command = #inner ;

                clapi::command_line::CommandLine::new(command)
                    .use_default_help()
                    .use_default_suggestions()
                    .run()
                    .expect("an error occurred");
            }
        }

        if self.is_child {
            inner
        } else {
            let name = self.name.as_str().parse::<TokenStream>().unwrap();
            let attrs = &self.item_fn.as_ref().unwrap().attrs;
            let outer = self.outer_body();
            //let ret = &self.item_fn.as_ref().unwrap().sig.output;

            quote! {
                #(#attrs)*
                fn #name() {
                    #inner
                    #(#outer)*
                }
            }
        }
    }

    fn get_body(&self, vars: &[TokenStream]) -> TokenStream {
        if self.is_child {
            let command_name = self.name.parse::<TokenStream>().unwrap();
            let input_vars = self.vars.iter().map(|var| {
                let name = var.name().parse::<TokenStream>().unwrap();
                let is_ref = match var.arg_type() {
                    ArgType::Slice(_) => quote! { & },
                    ArgType::MutSlice(_) => quote! { &mut },
                    _ => quote! {},
                };

                quote! { #is_ref #name }
            });

            quote! {
                #(#vars)*
                #command_name(#(#input_vars,)*);
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

impl ToTokens for CommandAttribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.expand().into_iter())
    }
}

pub struct AttributeKeys;
impl AttributeKeys {
    const OPTION: &'static str = "option";
    const ARG: &'static str = "arg";
    const SUBCOMMAND: &'static str = "subcommand";
    const DESCRIPTION: &'static str = "description";
    const ALIAS: &'static str = "alias";
    const MIN: &'static str = "min";
    const MAX: &'static str = "max";
    const DEFAULT: &'static str = "default";
}

fn new_command_tokens(attr_data: AttributeData, item_fn: ItemFn, is_child: bool) -> CommandAttribute {
    let name = item_fn.sig.ident.to_string();
    let mut command = if is_child {
        CommandAttribute::new_child(name)
    } else {
        CommandAttribute::new(name)
    };

    // Sets the body
    // command.set_item_fn(item_fn.clone());

    for (key, value) in &attr_data {
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
            _ => panic!("invalid {} key `{}`", attr_data.path(), key),
        }
    }

    let attrs = item_fn
        .attrs
        .iter()
        .cloned()
        .map(|att| (att.clone(), AttributeData::new(att)))
        .filter(|(_, n)| n.path() == "option" || n.path() == "arg")
        .collect::<Vec<(Attribute, AttributeData)>>();

    // Check all attributes have the `name` key declared, is a literal and is not empty
    assert_attributes_name_is_declared(&attrs);

    let named_fn_args = item_fn
        .sig
        .inputs
        .iter()
        .cloned()
        .map(|fn_arg| {
            if let FnArg::Typed(pat_type) = &fn_arg {
                (pat_type.pat.to_token_stream().to_string(), fn_arg.clone())
            } else {
                unreachable!("FnArg is not a free function")
            }
        })
        .collect::<Vec<(String, FnArg)>>();

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
            let command_args = ArgAttribute::from_attribute_data(fn_arg.attr.clone().unwrap(), Some(fn_arg.pat_type.clone()));
            command.set_args(command_args);
        } else {
            let mut command_args = ArgAttribute::new(None);
            command_args.set_max(arg_count);
            command.set_args(command_args);
        }
    }

    // Add options
    for arg in fn_args.iter().filter(|n| n.is_option) {
        let mut option = OptionAttribute::new(arg.arg_name.clone());
        let mut args = ArgAttribute::new(Some(arg.pat_type.clone()));

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
                        Value::Literal(_) => {
                            let s = value.clone().parse_literal::<String>().unwrap();
                            args.set_default_values(vec![s])
                        }
                        Value::Array(_) => {
                            let array = value.clone().parse_array::<String>().unwrap();
                            args.set_default_values(array)
                        }
                        _ => panic!("option `default` expected to be literal or array"),
                    },
                    _ => panic!("invalid {} key `{}`", att.path(), key),
                }
            }
        }

        option.set_args(args);
        command.set_option(option);
    }

    // Add children
    for (attr_data, item_fn) in get_children(&item_fn.block) {
        command.set_child(new_command_tokens(attr_data, item_fn, true));
    }

    command.set_fn(remove_fn_command_attributes(item_fn));
    command
}

#[derive(Debug)]
struct CommandFnArg {
    arg_name: String,
    pat_type: PatType,
    attr: Option<AttributeData>,
    is_option: bool,
}

fn get_children(block: &Block) -> Vec<(AttributeData, ItemFn)> {
    let mut ret = Vec::new();

    for stmt in &block.stmts {
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
                        subcommands.len(), 1,
                        "multiples `subcommand` attributes defined"
                    );

                    let mut item_fn = item_fn.clone();

                    let attr_data = if let Some(index) = item_fn.attrs
                        .iter()
                        .position(|att| att.path.to_token_stream().to_string() == "subcommand")
                    {
                        AttributeData::new(item_fn.attrs.swap_remove(index))
                    } else {
                        unreachable!()
                    };

                    ret.push((attr_data, item_fn))
                }
            }
        }
    }

    ret
}

fn assert_attributes_name_is_declared(fn_attrs: &[(Attribute, AttributeData)]) {
    for (att, data) in fn_attrs {
        if let Some(value) = data.get("name") {
            assert!(value.is_str(), "`name` must be a string literal");
            assert!(
                !value.as_string_literal().unwrap().is_empty(),
                "`name` cannot be empty"
            );
        } else {
            panic!(
                "{} `name` is required in `{}`",
                data.path(),
                att.to_token_stream().to_string()
            );
        }
    }
}

fn get_fn_args(
    attrs: &[(Attribute, AttributeData)],
    fn_args: &[(String, FnArg)],
) -> Vec<CommandFnArg> {
    let mut ret = Vec::new();
    //println!("{:?}", fn_args.iter().map(|n| n.0.clone()).collect::<Vec<String>>());

    // Checks attributes key `name` match a function arg
    for (attr, data) in attrs {
        match data.get("name").unwrap() {
            Value::Literal(lit) => {
                let name = literal_to_string(lit);

                let contains = fn_args
                    .iter()
                    .any(|(arg_name, _)| arg_name == name.as_str());

                assert!(
                    contains,
                    "cannot find arg named `{}` for `{}`",
                    name,
                    attr.to_token_stream().to_string()
                );
            }
            _ => panic!("expected string literal for `name`"),
        }
    }

    for (arg_name, fn_arg) in fn_args {
        if let FnArg::Typed(pat_type) = fn_arg.clone() {
            let arg_name = arg_name.clone();
            let attr = attrs
                .iter()
                .find(|(_, data)| {
                    data.get("name")
                        .map(|v| v.as_string_literal())
                        .flatten()
                        .contains_some(&arg_name)
                })
                .map(|n| n.1.clone());

            let is_option = match &attr {
                None => true,
                Some(att) => att.path() == "option",
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

fn get_fn_arg_ident_name(fn_arg: &FnArg) -> String {
    if let FnArg::Typed(pat_type) = fn_arg {
        if let Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
            return pat_ident.ident.to_string();
        }
    }

    unreachable!("Cannot get `FnArg` ident name")
}

fn remove_fn_command_attributes(mut item_fn: ItemFn) -> ItemFn {
    item_fn.attrs = item_fn.attrs.iter()
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

/*
#[command(description="A program")]
#[option(name="x", alias="n", description="a number")]
#[arg(name="args")
fn main(number: u32, enable: bool, args: Vec<String>){
    const NUMBER : u32 = 42;
    static ID : &'static str = "Hello World";

    struct Other;

    fn some_fun(){}

    #[subcommand(description="Create an app")]
    #[option(name="path", alias="p")]
    fn create(path: String){
        println!("{}", path);
    }

    println!("{}, {}", number, enable);
}

fn main(){
    fn create(path: String){
        println!("{}", path);
    }

    let root = RootCommand::new()
        .set_description("A program")
        .set_args(Arguments::new(0..))
        .set_option(CommandOption::new("number")
            .set_alias("n")
            .set_description("a number"))
        .set_option(CommandOption::new("enable"))
        .set_command(Command::new("create")
            .set_option(CommandOption::new("path")
                .set_alias("p"))
            .set_handler(|args, opts|{
                let path = opts.get_args("path").unwrap().convert_at::<String>(0).unwrap();
                create(path);
                Ok(())
            })
        .set_handler(|args, opts|{
            let number = opts.get_args("number").unwrap().convert_at::<u32>(0).unwrap();
            let enable = opts.get_args("enable").unwrap().convert_at::<enable>(0).unwrap();
            println!("{}, {}", number, enable);
            Ok(())
        });
}
*/
