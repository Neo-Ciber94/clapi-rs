use std::cell::{RefCell, RefMut};
use syn::{AttributeArgs, ItemFn};
use std::path::{PathBuf, Path};

pub struct CommandRawData {
    raw_attr_args: String,
    raw_item_fn: String,
    path: PathBuf
}

impl CommandRawData {
    pub fn new(raw_attr_args: String, raw_item_fn: String, path: PathBuf) -> Self {
        CommandRawData {
            raw_attr_args,
            raw_item_fn,
            path
        }
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn parse_args(&self) -> syn::Result<AttributeArgs> {
        use std::str::FromStr;;
        let stream = proc_macro2::TokenStream::from_str(&self.raw_attr_args).unwrap();
        syn::parse_macro_input::parse::<AttributeArgs>(stream.into())
    }

    pub fn parse_item_fn(&self) -> syn::Result<ItemFn> {
        use syn::parse::Parse;
        syn::parse::Parser::parse_str(ItemFn::parse, &self.raw_item_fn)
    }
}

pub fn get_subcommand_registry() -> RefMut<'static, Vec<CommandRawData>> {
    use std::sync::Once;
    use std::ptr::null_mut;

    static mut SUBCOMMANDS : *mut RefCell<Vec<CommandRawData>> = null_mut();
    static INIT : Once = Once::new();

    unsafe {
        INIT.call_once(|| {
            SUBCOMMANDS = Box::into_raw(Box::new(RefCell::new(Vec::new())));
        });

        (*SUBCOMMANDS).borrow_mut()
    }
}

pub fn add_subcommand(command: CommandRawData) {
    get_subcommand_registry().push(command)
}