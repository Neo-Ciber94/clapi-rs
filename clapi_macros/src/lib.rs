#[allow(unused_variables)]
#[allow(dead_code)]

extern crate proc_macro;
use proc_macro::TokenStream;
use quote::*;
use syn::*;

#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = syn::parse_macro_input!(item as syn::ItemFn);
    let args = syn::parse_macro_input!(attr as syn::AttributeArgs);

    insert_command_to_func(args, func)
}

fn insert_command_to_func(args: AttributeArgs, func: ItemFn) -> TokenStream{
    get_func_params(&func);

    let func_name = func.sig.ident;
    let func_body = func.block;

    let tokens = quote! {
        fn #func_name(){
            #func_body
            println!("Hello World");
        }
    };

    tokens.into()
}

struct FnParam{
    name: String,
    ty: Box<Type>
}

fn get_func_params(func: &ItemFn) -> Vec<FnParam>{
    let mut result = Vec::new();

    for fn_arg in &func.sig.inputs {
        if let FnArg::Typed(pat_type) = &fn_arg{
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                let name = pat_ident.ident.to_string();
                let ty = pat_type.ty.clone();
                result.push(FnParam{ name, ty });
            }
        }
    }

    result
}