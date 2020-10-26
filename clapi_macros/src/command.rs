use crate::args::ArgsTokens;
use crate::attr_data::AttributeData;
use crate::option::OptionTokens;
use crate::var::LocalVar;
use clapi::utils::OptionExt;
use proc_macro2::TokenStream;
use syn::export::ToTokens;
use syn::{Attribute, AttributeArgs, Block, FnArg, ItemFn, PatType, PathArguments, Signature};

/// Tokens for:
///
/// ```ignore
/// #[command(
///     description="A description",
///     help="Help text",
/// )]
/// ```
pub struct CommandTokens {
    //CommandAttribute, CommandAttr
    name: String,
    description: Option<String>,
    help: Option<String>,
    body: Option<Box<Block>>,
    children: Vec<CommandTokens>,
    options: Vec<OptionTokens>,
    args: Option<ArgsTokens>,
    vars: Vec<LocalVar>,
}

#[allow(dead_code)]
#[allow(unused_variable)]
impl CommandTokens {
    pub fn new(name: String) -> Self {
        CommandTokens {
            name,
            description: None,
            help: None,
            body: None,
            children: vec![],
            options: vec![],
            vars: vec![],
            args: None,
        }
    }

    pub fn from_attribute_args(att: AttributeArgs, func: ItemFn) -> Self {
        new_command_tokens(att, func)
    }

    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
    }

    pub fn set_help(&mut self, help: String) {
        self.help = Some(help);
    }

    pub fn set_child(&mut self, command: CommandTokens) {
        self.children.push(command)
    }

    pub fn set_option(&mut self, option: OptionTokens) {
        self.options.push(option);
    }

    pub fn set_args(&mut self, args: ArgsTokens, var: LocalVar) {
        self.args = Some(args);
        self.vars.push(var);
    }

    pub fn set_var(&mut self, var: LocalVar) {
        self.vars.push(var);
    }

    pub fn set_body(&mut self, body: Box<Block>) {
        self.body = Some(body);
    }

    pub fn expand(&self) -> TokenStream {
        todo!()
    }
}

fn new_command_tokens(args: AttributeArgs, func: ItemFn) -> CommandTokens {
    let name = func.sig.ident.to_string();
    let mut root = CommandTokens::new(name.clone());
    let attr = AttributeData::from_attribute_args(name, args);

    // Sets the body
    root.set_body(func.block.clone());

    // Sets the description if any
    if let Some(value) = attr.get("description").cloned() {
        let description = value
            .into_literal()
            .expect("`description` is expected to be string literal");
        root.set_description(description);
    }

    // Sets the help if any
    if let Some(value) = attr.get("help").cloned() {
        let help = value
            .into_literal()
            .expect("`help` is expected to be string literal");
        root.set_description(help);
    }

    let attrs = func
        .attrs
        .iter()
        .cloned()
        .map(|att| (att.clone(), AttributeData::new(att)))
        .filter(|(_, n)| n.path() == "option" || n.path() == "args")
        .collect::<Vec<(Attribute, AttributeData)>>();

    //let fn_args = get_fn_args(&func.sig);

    // Check all attributes have the `name` key declared, is a literal and is not empty
    assert_attributes_name_is_declared(&attrs);

    // Check all the attributes have an function arg that match the `name` key
    //assert_attributes_match_arg(&attrs, &fn_args);


    root
}

struct CommandFnArg {
    arg_name: String,
    pat_type: PatType,
    attr: Option<AttributeData>,
}

fn assert_attributes_name_is_declared(fn_attrs: &[(Attribute, AttributeData)]) {
    for (att, data) in fn_attrs {
        if let Some(value) = data.get("name") {
            assert!(value.is_literal(), "`name` must be a string literal");
            assert!(
                !value.as_literal().unwrap().is_empty(),
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

fn get_fn_args(attrs: &[(Attribute, AttributeData)], fn_args: &[FnArg]) -> Vec<CommandFnArg> {
    let mut ret = Vec::new();
    let fn_args = fn_args.iter()
        .cloned()
        .map(|n| (n.to_token_stream().to_string(), n))
        .collect::<Vec<(String, FnArg)>>();

    // Checks attributes `name` key match a function arg
    for (attr, data) in attrs {
        let name = data.get("name").unwrap().as_literal().unwrap();
        let contains_arg = fn_args.iter().any(|(arg_name, _)| arg_name == name);
        assert!(contains_arg, "cannot find arg named `{}`: `{}`", name, attr.to_token_stream().to_string());
    }

    for (arg_name, fn_arg) in fn_args {
        if let FnArg::Typed(pat_type) = fn_arg.clone() {
            let attr = attrs
                .iter()
                .find(|(_, data)| {
                    data.get("name")
                        .unwrap()
                        .as_literal()
                        .contains_some(&arg_name)
                })
                .map(|n| n.1.clone());

            ret.push(CommandFnArg { arg_name, pat_type, attr, });
        }
    }

    ret
}