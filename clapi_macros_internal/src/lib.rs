use proc_macro::TokenStream;

mod ext;
mod var;
pub(crate) use var::*;

// This is used internally in `app_macros` to generate the options variables
#[proc_macro]
pub fn __declare_option_var(input: TokenStream) -> TokenStream {
    let var_input = syn::parse_macro_input!(input as VarInput);
    DeclareVar::new(var_input, VarSource::Option)
        .expand()
        .into()
}

// This is used internally in `app_macros` to generate the argument variables
#[proc_macro]
pub fn __declare_argument_var(input: TokenStream) -> TokenStream {
    let var_input = syn::parse_macro_input!(input as VarInput);
    DeclareVar::new(var_input, VarSource::Argument)
        .expand()
        .into()
}

#[proc_macro]
pub fn __var_type(input: TokenStream) -> TokenStream {
    use syn::export::ToTokens;
    use syn::Type;

    let input_type = match syn::parse_macro_input!(input as Type) {
        Type::Group(ty) => ty.elem.as_ref().clone(),
        ty => ty,
    };

    get_var_type(&input_type).inner().to_token_stream().into()
}
