use crate::args::ArgsTokens;
use crate::option::OptionTokens;
use proc_macro2::TokenStream;
use syn::{AttributeArgs, Block, ItemFn, Signature, PatType, FnArg, Type};
use crate::attr_data::{AttributeData, Value};
use clapi::symbol::Symbol::Command;
use clapi::utils::Also;
use syn::export::ToTokens;

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
    options: Vec<OptionTokens>,
    args: Option<ArgsTokens>,
    body: Option<Box<Block>>,
}

#[allow(dead_code)]
#[allow(unused_variable)]
impl CommandTokens {
    pub fn new(name: String) -> Self{
        CommandTokens{
            name,
            description: None,
            help: None,
            children: vec![],
            options: vec![],
            args: None,
            body: None
        }
    }

    pub fn from_attribute_args(att: AttributeArgs, func: ItemFn) -> Self {
        create_command_tokens(att, func)
    }

    pub fn set_description(&mut self, description: String){
        self.description = Some(description);
    }

    pub fn set_help(&mut self, help: String){
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

    fn set_body(&mut self, body: Box<Block>){
        self.body = Some(body);
    }

    pub fn expand(&self) -> TokenStream {
        todo!()
    }
}

fn create_command_tokens(args: AttributeArgs, func: ItemFn) -> CommandTokens{
    let name = func.sig.ident.to_string();
    let mut root = CommandTokens::new(name.clone());
    let attr = AttributeData::from_attribute_args(name, args);

    // Sets the body
    root.set_body(func.block.clone());

    // Sets the description if any
    if let Some(description) = attr.get("description")
        .cloned()
        .map(|v| {
            match v {
                Value::Literal(l) => l,
                _ => panic!("`description` is expected to be string literal")
            }
        }){
        root.set_description(description);
    }

    // Sets the help if any
    if let Some(help) = attr.get("help")
        .cloned()
        .map(|v| {
            match v {
                Value::Literal(l) => l,
                _ => panic!("`help` is expected to be string literal")
            }
        }){
        root.set_help(help);
    }

    let mut fn_attrs = func.attrs.iter()
        .cloned()
        .map(AttributeData::new)
        .collect::<Vec<AttributeData>>();

    //assert!(fn_attrs.iter().filter(|f| f.path() == "args").count() <= 1, "multiple `args` are defined");

    let mut fn_args = get_fn_args(&func.sig);

    // Sets root command args if any
    if let Some(att_index) = fn_attrs.iter().position(|a| a.path() == "args"){
        let attr = fn_attrs.swap_remove(att_index);
        let arg_index = fn_args.iter()
            .position(|n| n.path == attr.path())
            .unwrap_or_else(||{
                let fn_name = func.sig.ident.to_string();
                panic!("arg named `{}` is not defined in `{}`", attr.path(), fn_name);
            });

        let typed_arg = fn_args.swap_remove(arg_index);
        root.set_args(ArgsTokens::from(typed_arg, Some(attr)));
    } else {
        if let Some(fn_arg) = fn_args.iter().find(|a| a.path == "args"){
            let mut args = ArgsTokens::new();
            args.set_arg_type(fn_arg.ty.clone());
            root.set_args(args);
        }
    }

    root
}

pub struct TypedFnArg {
    pub path: String,
    pub ty: Box<Type>
}

impl TypedFnArg {
    pub fn is_slice(&self) -> bool {
        todo!()
    }

    pub fn is_vec(&self) -> bool{
        todo!()
    }

    pub fn ty(&self) -> Box<Type>{
        todo!()
    }

    pub fn expand_let(&self) -> TokenStream{
        /*
        if slice
        let mut #path = result.get_option_arsg(#path).args::<#ty>();
        */
        todo!()
    }
}

fn get_fn_args(sig: &Signature) -> Vec<TypedFnArg>{
    let mut ret = Vec::new();

    for input in &sig.inputs {
        if let FnArg::Typed(pat_type) = input {
            let name = pat_type.pat.to_token_stream().to_string();
            let ty = pat_type.ty.clone();
            ret.push(TypedFnArg { path: name, ty });
        }
    }

    ret
}