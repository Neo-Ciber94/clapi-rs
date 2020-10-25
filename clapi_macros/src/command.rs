use crate::args::ArgsTokens;
use crate::attr_data::{AttributeData, Value};
use crate::option::OptionTokens;
use crate::var::LocalVar;
use clapi::symbol::Symbol::Command;
use clapi::utils::Also;
use proc_macro::Ident;
use proc_macro2::TokenStream;
use syn::export::ToTokens;
use syn::{
    Attribute, AttributeArgs, Block, FnArg, GenericArgument, ItemFn, Pat, PatIdent, PatType,
    PathArguments, Signature, Type,
};

/// Tokens for:
///
/// ```ignore
/// #[command(
///     description="A description",
///     help="Help text",
/// )]
/// ```
pub struct CommandTokens {
    name: String,
    description: Option<String>,
    help: Option<String>,
    children: Vec<CommandTokens>,
    vars: Vec<LocalVar>,
    options: Vec<OptionTokens>,
    args: Option<ArgsTokens>,
    body: Option<Box<Block>>,
}

#[allow(dead_code)]
#[allow(unused_variable)]
impl CommandTokens {
    pub fn new(name: String) -> Self {
        CommandTokens {
            name,
            description: None,
            help: None,
            children: vec![],
            vars: vec![],
            options: vec![],
            args: None,
            body: None,
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
        self.options.push(option)
    }

    pub fn set_args(&mut self, args: ArgsTokens) {
        self.args = Some(args)
    }

    fn set_body(&mut self, body: Box<Block>) {
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
    if let Some(value) = attr.get("description").cloned(){
        let description = value.into_literal().expect("`description` is expected to be string literal");
        root.set_description(description);
    }

    // Sets the help if any
    if let Some(value) = attr.get("help").cloned(){
        let help = value.into_literal().expect("`help` is expected to be string literal");
        root.set_description(help);
    }

    let mut fn_attrs = func
        .attrs
        .iter()
        .cloned()
        .map(|att| (att.clone(), AttributeData::new(att)))
        .collect::<Vec<(Attribute, AttributeData)>>();

    assert_valid_attributes(&fn_attrs);
    assert_attributes_name_is_declared(&fn_attrs);

    struct FnTypeArg{
        path: String,
        ty: Box<Type>,
        attr: Option<AttributeData>
    }

    //let mut fn_type_args = Vec::new();

    for pat_type in get_fn_args(&func.sig) {
        let path = pat_type.pat.to_token_stream().to_string();
        let ty = pat_type.ty;

        //fn_type_args.push(TypedFnArg{ path, ty, attr });
    }

    //assert!(fn_attrs.iter().filter(|f| f.path() == "args").count() <= 1, "multiple `args` are defined");

    //let mut fn_args = get_fn_args(&func.sig);

    // Sets root command args if any
    // if let Some(att_index) = fn_attrs.iter().position(|a| a.path() == "args"){
    //     let attr = fn_attrs.swap_remove(att_index);
    //     let arg_index = fn_args.iter()
    //         .position(|n| n.path == attr.path())
    //         .unwrap_or_else(||{
    //             let fn_name = func.sig.ident.to_string();
    //             panic!("arg named `{}` is not defined in `{}`", attr.path(), fn_name);
    //         });
    //
    //     let typed_arg = fn_args.swap_remove(arg_index);
    //     root.set_args(ArgsTokens::from(typed_arg, Some(attr)));
    // } else {
    //     if let Some(fn_arg) = fn_args.iter().find(|a| a.path == "args"){
    //         let mut args = ArgsTokens::new();
    //         args.set_arg_type(fn_arg.ty.clone());
    //         root.set_args(args);
    //     }
    // }

    root
}

fn assert_attributes_name_is_declared(fn_attrs: &Vec<(Attribute, AttributeData)>) {
    for (att, data) in fn_attrs.iter() {
        if let Some(value) = data.get("name"){
            assert!(value.is_literal(), "`name` must be a string literal");
        } else {
            panic!("{} `name` is required in `{}`", data.path(), att.to_token_stream().to_string());
        }
    }
}

fn assert_valid_attributes(fn_attrs: &Vec<(Attribute, AttributeData)>) {
    for (att, data) in fn_attrs.iter() {
        assert!(
            data.path() == "option" || data.path() == "args",
            "invalid attribute: `{}`\nexpected `#[option(...)]` or `#[args(...)]`",
            att.to_token_stream().to_string()
        );
    }
}

pub struct TypedFnArg {
    pub path: String,
    pub ty: Box<Type>,
}

fn get_typed_fn_args(sig: &Signature) -> Vec<TypedFnArg> {
    let mut ret = Vec::new();

    for input in &sig.inputs {
        if let FnArg::Typed(pat_type) = input {
            let path = pat_type.pat.to_token_stream().to_string();
            let ty = pat_type.ty.clone();
            ret.push(TypedFnArg { path, ty });
        }
    }

    ret
}

fn get_fn_args(sig: &Signature) -> Vec<PatType> {
    let mut ret = Vec::new();

    for input in &sig.inputs {
        if let FnArg::Typed(pat_type) = input {
            ret.push(pat_type.clone());
        }
    }

    ret
}
