#[allow(unused_variables)]
#[allow(dead_code)]

extern crate proc_macro;
use proc_macro::TokenStream;
use quote::*;
use syn::*;
use syn::export::quote::__private::ext::RepToTokensExt;
use proc_macro::TokenTree::Literal;

/*
#[command(description="", help="")]

#[option(name="", alias="", description="", default="")]

#[args(name="x", default="")]
*/

/*
#[command(description="A description", help="A help")]
#[option(name="x", alias="X", description="A number", default=0)]
#[args(default=["one", "two", "three", min=0, max=3])]
fn main(x: u32, y: String, z: bool, args: Vec<String>){

}

#[command]
fn test(args: Vec<u32>){
}

converts to:
fn main(){
    let root = RootCommand::new()
        .set_description("A description")
        .set_help("A help")
        .set_args(Arguments::new(0..=3)
            .set_name("args")
            .set_default_values(&["one", "two", "three"]))
        .set_option(CommandOption::new("x")
            .set_alias("X")
            .set_description("A number")
            .set_args(Arguments::new(1).set_default_values(&[0]))
        .set_command(Command::new("test")
            .set_handler(|opts, args|){
                let args = args.convert_all::<u32>();
                // fn create body
            }));

     let context = Context::new(root);
     let mut parser = DefaultParser::default();
     let result = parser.parse(context, std::env::args().skip(1)).unwrap();

     let x = result.get_option_arg_as::<u32>("x").unwrap();
     let y = result.get_option_arg_as::<String>("y").unwrap();
     let z = result.get_option_arg_as::<bool>("z").unwrap();
     let args = result.args().convert_all::<String>().unwrap();
}
*/


#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {

    let var_name = parse_str("let x = 10;");

    let tokens = quote! {
        fn main(){
            #var_name
            println!("The value is: {}", x);
        }
    };

    tokens.into()
}

fn parse_str(value: &str) -> proc_macro2::TokenStream{
    value.parse().unwrap()
}

struct FnParam{
    name: Ident,
    ty: Box<Type>
}

fn get_func_params(func: &ItemFn) -> Vec<FnParam>{
    let mut result = Vec::new();

    for fn_arg in &func.sig.inputs {
        if let FnArg::Typed(pat_type) = &fn_arg{
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                let name = pat_ident.ident.clone();
                let ty = pat_type.ty.clone();
                result.push(FnParam{ name, ty });
            }
        }
    }

    result
}

enum AttributeKind{
    Command(String),
    Option(String),
    Arg(String),
}

struct CommandFromTokens;
struct OptionFromTokens;
struct ArgsFromTokens;