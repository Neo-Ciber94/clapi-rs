use std::path::PathBuf;

use proc_macro2::TokenStream;
use quote::*;
use syn::export::fmt::Display;
use syn::export::{Formatter, ToTokens};
use syn::{AttrStyle, Attribute, AttributeArgs, Item, ItemFn, PatType, ReturnType, Stmt, Type};

use crate::arg::ArgAttrData;
use crate::attr;
use crate::macro_attribute::NameValueAttribute;
use crate::option::OptionAttrData;
use crate::var::{ArgLocalVar, ArgumentType};
use crate::TypeExtensions;

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
#[derive(Debug)]
pub struct CommandAttrData {
    fn_name: FnName,
    is_child: bool,
    version: Option<String>,
    description: Option<String>,
    about: Option<String>,
    item_fn: Option<ItemFn>,
    children: Vec<CommandAttrData>,
    options: Vec<OptionAttrData>,
    args: Option<ArgAttrData>,
    vars: Vec<ArgLocalVar>,
}

impl CommandAttrData {
    fn new(name: FnName, is_child: bool) -> Self {
        CommandAttrData {
            fn_name: name,
            is_child,
            version: None,
            description: None,
            about: None,
            item_fn: None,
            children: vec![],
            options: vec![],
            vars: vec![],
            args: None,
        }
    }

    pub fn from_fn(args: AttributeArgs, func: ItemFn) -> Self {
        let name = func.sig.ident.to_string();
        let attr_data =
            NameValueAttribute::from_attribute_args(name.as_str(), args, AttrStyle::Outer).unwrap();
        cmd::new_command(attr_data, func, false, true)
    }

    pub fn from_path(args: AttributeArgs, func: ItemFn, path: PathBuf) -> Self {
        cmd::new_command_from_path(args, func, path)
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
        self.children.push(command)
    }

    pub fn set_option(&mut self, option: OptionAttrData) {
        self.options.push(option);
    }

    pub fn set_args(&mut self, args: ArgAttrData) {
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
            self.fn_name
        );

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

        // Command args
        let args = self
            .args
            .as_ref()
            .map(|tokens| quote! { .arg(#tokens)})
            .unwrap_or_else(|| quote! {});

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

        // Build the command
        command = quote! {
            #command
                #description
                #about
                #args
                #version
                #(#options)*
                #(#children)*
                .handler(|opts, args|{
                    #show_version
                    #body
                })
        };

        if self.is_child {
            command
        } else {
            let name = self.fn_name.name().parse::<TokenStream>().unwrap();
            let ret = &self.item_fn.as_ref().unwrap().sig.output;
            let attrs = &self.item_fn.as_ref().unwrap().attrs;
            let outer = self.outer_body();
            let error_handling = match ret {
                ReturnType::Type(_, ty) if is_clapi_result_type(ty) => quote! {},
                _ => quote! { .expect("an error occurred"); },
            };

            // Emit the tokens to create the function with the `RootCommand`
            quote! {
                #(#attrs)*
                fn #name() #ret {
                    #(#outer)*

                    let command = #command ;
                    clapi::CommandLine::new(command)
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
            ReturnType::Type(_, ty) if is_clapi_result_type(ty) => quote! {},
            // If return type is not `Result` we need return `fn_name(args) ; Ok(())`
            _ => quote! { ; Ok(()) },
        };

        if self.is_child {
            let fn_name = self.fn_name.to_string().parse::<TokenStream>().unwrap();
            let inputs = self.vars.iter().map(|var| {
                let var_name = var.name().parse::<TokenStream>().unwrap();
                let is_ref = match var.arg_type() {
                    ArgumentType::Slice(_) => quote! { & },
                    ArgumentType::MutSlice(_) => quote! { &mut },
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
        /// If the `Stmt` is a function, drop all the command attributes
        fn drop_attributes_if_subcommand(stmt: Stmt) -> Stmt {
            if let Stmt::Item(Item::Fn(item_fn)) = stmt {
                return Stmt::Item(Item::Fn(drop_command_attributes(item_fn)));
            }

            stmt
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
            .map(drop_attributes_if_subcommand)
            .map(|s| s.to_token_stream())
            .collect()
    }
}

impl ToTokens for CommandAttrData {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.expand().into_iter())
    }
}

#[derive(Debug, Clone)]
pub struct FnArgData {
    pub arg_name: String,
    pub pat_type: PatType,
    pub attribute: Option<NameValueAttribute>,
    pub is_option: bool,
}

impl FnArgData {
    pub fn drop_attribute(mut self) -> Self {
        self.attribute = None;
        self
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FnName {
    path: Option<String>,
    name: String,
}

impl FnName {
    pub fn new(path: Option<String>, name: String) -> Self {
        FnName { path, name }
    }

    pub fn path(&self) -> Option<&str> {
        self.path.as_ref().map(|s| s.as_str())
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl Display for FnName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(path) = &self.path {
            write!(f, "{}::{}", path, self.name)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

fn is_clapi_result_type(ty: &Type) -> bool {
    if ty.is_result() {
        return true;
    }

    match ty.path().unwrap().as_str() {
        "clapi::Result" | "clapi::error::Result" => true,
        _ => false,
    }
}

pub fn drop_command_attributes(mut item_fn: ItemFn) -> ItemFn {
    item_fn.attrs = item_fn
        .attrs
        .iter()
        .filter(|att| {
            let path = att.path.to_token_stream().to_string();
            !attr::is_clapi_attribute(&path)
        })
        .cloned()
        .collect::<Vec<Attribute>>();

    item_fn
}

pub fn is_option_bool_flag(fn_arg: &FnArgData) -> bool {
    if let Some(attr) = &fn_arg.attribute {
        fn_arg.pat_type.ty.is_bool()
            && !(attr.contains_name(attr::MIN)
                || attr.contains_name(attr::MAX)
                || attr.contains_name(attr::DEFAULT))
    } else {
        fn_arg.pat_type.ty.is_bool()
    }
}

mod cmd {
    use std::convert::TryFrom;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, Ordering};

    use syn::{
        AttrStyle, Attribute, AttributeArgs, File, FnArg, Item, ItemFn, ItemMod, PatType, Stmt,
        Type,
    };

    use crate::arg::ArgAttrData;
    use crate::command::{
        drop_command_attributes, is_option_bool_flag, CommandAttrData, FnArgData, FnName,
    };
    use crate::macro_attribute::{MacroAttribute, MetaItem, NameValueAttribute};
    use crate::option::OptionAttrData;
    use crate::utils::{pat_type_to_string, path_to_string};
    use crate::var::{ArgLocalVar, VarSource};
    use crate::TypeExtensions;
    use crate::{attr, AttrQuery};

    // Create a new command from an `ItemFn`

    pub fn new_command(
        name_value_attr: NameValueAttribute,
        item_fn: ItemFn,
        is_child: bool,
        get_subcommands: bool,
    ) -> CommandAttrData {
        let name = item_fn.sig.ident.to_string();
        new_command_with_name(
            name_value_attr,
            item_fn,
            FnName::new(None, name),
            is_child,
            get_subcommands,
        )
    }

    pub fn new_command_with_name(
        name_value_attr: NameValueAttribute,
        item_fn: ItemFn,
        name: FnName,
        is_child: bool,
        get_subcommands: bool,
    ) -> CommandAttrData {
        let mut command = CommandAttrData::new(name, is_child);

        for (key, value) in &name_value_attr {
            match key.as_str() {
                attr::DESCRIPTION => {
                    let description = value
                        .clone()
                        .to_string_literal()
                        .expect("`description` must be a string literal");
                    command.set_description(description);
                }
                attr::ABOUT => {
                    let help = value
                        .clone()
                        .to_string_literal()
                        .expect("`about` must be a string literal");
                    command.set_description(help);
                }
                attr::VERSION => {
                    assert!(
                        value.is_integer() || value.is_float() || value.is_string(),
                        "`version` must be an integer, float or string literal"
                    );
                    command.set_version(value.parse_literal::<String>().unwrap());
                }
                _ => panic!("invalid `{}` key `{}`", name_value_attr.path(), key),
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
                if arg_count > 1 {
                    let ty = fn_arg.pat_type.ty.as_ref();
                    if ty.is_slice() || ty.is_vec() {
                        panic!("invalid argument type for: `{}`\
                        \nwhen multiples `arg` are defined, arguments cannot be declared as `Vec` or `slice`",
                          pat_type_to_string(&fn_arg.pat_type));
                    }

                    if let Some(attr) = &fn_arg.attribute {
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
                    VarSource::Args(fn_arg.arg_name.clone()),
                ));
            }
        }

        // Add args
        if arg_count > 0 {
            for fn_arg in fn_args.iter().filter(|f| !f.is_option) {
                let arg = if fn_arg.attribute.is_some() {
                    ArgAttrData::from_arg_data(fn_arg.clone())
                } else {
                    ArgAttrData::with_name(fn_arg.arg_name.clone())
                };

                command.set_args(arg)
            }
        }

        // Add options
        for fn_arg in fn_args.into_iter().filter(|n| n.is_option) {
            let option = OptionAttrData::from_arg_data(fn_arg);
            command.set_option(option);
        }

        // Add children
        if get_subcommands {
            for (name_value, item_fn) in get_subcommands_from_fn(&item_fn) {
                let subcommand = new_command(name_value, item_fn, true, true);
                command.set_child(subcommand);
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
                        .filter(|att| attr::is_subcommand(path_to_string(&att.path).as_str()))
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
                                attr::is_subcommand(path_to_string(&att.path).as_str())
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

            unreachable!("No a function arg")
        }

        let mut ret = Vec::new();
        let attributes = item_fn
            .attrs
            .iter()
            .cloned()
            .map(|att| MacroAttribute::new(att))
            .filter(|att| attr::is_option(att.path()) || attr::is_arg(att.path()))
            .map(|att| split_attr_path_and_name_values(&att))
            .collect::<Vec<(String, NameValueAttribute)>>();

        let fn_args = item_fn
            .sig
            .inputs
            .iter()
            .map(|f| get_fn_arg_ident_name(f))
            .collect::<Vec<(String, PatType)>>();

        for (arg_name, pat_type) in fn_args {
            let attribute = attributes.iter().find_map(|(path, att)| {
                if path == &arg_name {
                    Some(att.clone())
                } else {
                    None
                }
            });

            let is_option = attribute
                .as_ref()
                .map(|att| attr::is_option(att.path()))
                .unwrap_or(true);

            ret.push(FnArgData {
                arg_name,
                pat_type,
                attribute,
                is_option,
            });
        }

        ret
    }

    fn split_attr_path_and_name_values(attr: &MacroAttribute) -> (String, NameValueAttribute) {
        let name = attr.get(0)
            .cloned()
            .unwrap_or_else(|| panic!("the first element in `{}` must be the argument name, but was empty", attr))
            .into_path()
            .unwrap_or_else(|| {
                panic!("first element in `{}` must be a path like: `#[{}(value, ...)]` where `value` is the name of the function argument", attr, attr.path())
            });

        let name_value_attribute = if attr.len() == 1 {
            NameValueAttribute::empty(attr.path().to_owned(), AttrStyle::Outer)
        } else {
            let meta_items = attr[1..].iter().cloned().collect::<Vec<MetaItem>>();
            NameValueAttribute::new(attr.path(), meta_items, AttrStyle::Outer).unwrap()
        };

        (name, name_value_attribute)
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

    // Create a new command from a path

    pub fn new_command_from_path(
        args: AttributeArgs,
        item_fn: ItemFn,
        root_path: PathBuf,
    ) -> CommandAttrData {
        static IS_DEFINED: AtomicBool = AtomicBool::new(false);

        if IS_DEFINED.load(Ordering::Relaxed) {
            panic!(
                "multiple `command` entry points defined: `{}`",
                item_fn.sig.ident
            );
        } else {
            IS_DEFINED.store(true, Ordering::Relaxed);
        }

        let src = std::fs::read_to_string(&root_path)
            .unwrap_or_else(|_| panic!("failed to read: {}", root_path.display()));
        let root_file = syn::parse_file(&src)
            .unwrap_or_else(|_| panic!("failed to parse: {:?}", root_path.file_name().unwrap()));

        // root `command` must be a top function
        crate::assertions::is_top_function(&item_fn, &root_file);

        let attr =
            NameValueAttribute::from_attribute_args(attr::COMMAND, args, AttrStyle::Outer).unwrap();
        let mut command = new_command(attr, item_fn.clone(), false, true);

        for (path, attr, item_fn, file) in find_subcommands(&root_path, &root_file) {
            // Subcommands must be top functions or inner functions
            crate::assertions::is_top_function(&item_fn, &file);

            if path == root_path {
                command.set_child(new_command(attr, item_fn, true, true));
            } else {
                let name = get_item_fn_path_name(&path, &item_fn);
                let subcommand = new_command_with_name(attr, item_fn, name, true, true);
                command.set_child(subcommand);
            }
        }

        command
    }

    fn get_item_fn_path_name(path: &Path, item_fn: &ItemFn) -> FnName {
        let mut ret = Vec::new();

        // Iterate from `src/`
        let iter = path
            .iter()
            .map(|s| s.to_str().unwrap())
            .skip_while(|s| *s != "src")
            .skip(1);

        for item in iter {
            // `mod.rs` is not necessary to access the module
            if item == "mod.rs" {
                break;
            }

            ret.push(item.trim_end_matches(".rs").to_string());
        }

        if ret.is_empty() {
            return FnName::new(None, item_fn.sig.ident.to_string());
        }

        FnName::new(Some(ret.join("::")), item_fn.sig.ident.to_string())
    }

    fn find_subcommands(
        root_path: &Path,
        root_file: &File,
    ) -> Vec<(PathBuf, NameValueAttribute, ItemFn, File)> {
        let mut subcommands = Vec::new();

        for (item_fn, path, file) in
            find_subcommands_item_fn_from_path_recursive(root_path, root_file)
        {
            let attr = get_subcommand_attribute(&item_fn).unwrap();
            subcommands.push((path, attr, item_fn, file))
        }

        subcommands
    }

    fn get_subcommand_attribute(item_fn: &ItemFn) -> Option<NameValueAttribute> {
        if let Some(att) = item_fn
            .attrs
            .iter()
            .find(|a| attr::is_subcommand(path_to_string(&a.path).as_str()))
        {
            Some(NameValueAttribute::try_from(att.clone()).unwrap())
        } else {
            None
        }
    }

    fn find_subcommands_item_fn_from_path(path: &Path) -> Vec<(ItemFn, File)> {
        let mut ret = Vec::new();
        let src = std::fs::read_to_string(path).unwrap();
        let file = syn::parse_file(&src).unwrap();

        for item in &file.items {
            match item {
                Item::Fn(item_fn) if item_fn.contains_attribute(attr::SUBCOMMAND) => {
                    ret.push((item_fn.clone(), file.clone()));
                }
                _ => {}
            }
        }

        ret
    }

    fn find_subcommands_item_fn_from_path_recursive(
        root_path: &Path,
        file: &File,
    ) -> Vec<(ItemFn, PathBuf, File)> {
        let mut ret = Vec::new();

        for item in &file.items {
            match item {
                // If the item is a module, find the `subcommands` in the files
                Item::Mod(item_mod) => {
                    // Find the `path` of the module, which is either a module path or a file
                    if let Some(path) = find_item_mod_path(root_path, item_mod) {
                        // If is a file find the `subcommands` and add all
                        if path.is_file() {
                            for (item_fn, file) in find_subcommands_item_fn_from_path(&path) {
                                ret.push((item_fn, path.clone(), file));
                            }
                        } else {
                            // If is a directory find the `mod.rs` to locate all the files
                            ret.extend(find_subcommands_item_fn_from_module_path(&path));
                        }
                    }
                }
                // If the item is a function, add it if is a `subcommand`
                Item::Fn(item_fn) => {
                    if item_fn.contains_attribute(attr::SUBCOMMAND) {
                        ret.push((item_fn.clone(), root_path.to_path_buf(), file.clone()));
                    }
                }
                _ => {}
            }
        }

        ret
    }

    fn find_subcommands_item_fn_from_module_path(path: &Path) -> Vec<(ItemFn, PathBuf, File)> {
        if let Ok(read_dir) = path.read_dir() {
            for e in read_dir {
                if let Ok(entry) = e {
                    // File path
                    let path = entry.path();
                    if path.is_file() && entry.file_name() == "mod.rs" {
                        let src = std::fs::read_to_string(entry.path()).unwrap();
                        let file = syn::parse_file(&src).unwrap();
                        return find_subcommands_item_fn_from_path_recursive(&path, &file);
                    }
                }
            }
        }

        Vec::new()
    }

    fn find_item_mod_path(root_path: &Path, mod_item: &ItemMod) -> Option<PathBuf> {
        if mod_item.content.is_some() {
            Some(root_path.to_path_buf())
        } else {
            let mod_name = mod_item.ident.to_string();
            find_item_mod_path_by_name(root_path, mod_name)
        }
    }

    fn find_item_mod_path_by_name(cur_path: &Path, mod_item_name: String) -> Option<PathBuf> {
        if cur_path.is_file() {
            if let Some(parent) = cur_path.parent() {
                return find_item_mod_path_by_name(parent, mod_item_name.clone());
            }
        }

        for dir in cur_path.read_dir().ok()? {
            if let Ok(entry) = dir {
                let path = entry.path();
                if path.is_file() {
                    if path.file_stem().unwrap() == mod_item_name.as_str() {
                        if path.extension().unwrap() != "rs" {
                            return None;
                        }
                        return Some(path);
                    }
                } else {
                    // Is is a directory and the name matches is a `mod`
                    if path.file_name().unwrap() == mod_item_name.as_str() {
                        return Some(path);
                    }
                    return find_item_mod_path_by_name(&path, mod_item_name.clone());
                }
            }
        }

        if let Some(parent) = cur_path.parent() {
            // Don't go after the `src/` parent
            if cur_path.file_name().unwrap() == "src" {
                return None;
            }

            return find_item_mod_path_by_name(parent, mod_item_name.clone());
        } else {
            None
        }
    }
}
