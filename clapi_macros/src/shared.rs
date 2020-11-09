use std::cell::{RefCell, RefMut};
use syn::{AttributeArgs, ItemFn};

pub struct CommandRawData {
    raw_attr_args: String,
    raw_item_fn: String,
}

impl CommandRawData {
    pub fn new(raw_attr_args: String, raw_item_fn: String) -> Self {
        CommandRawData {
            raw_attr_args,
            raw_item_fn
        }
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

static mut SUBCOMMANDS : Option<RefCell<Vec<CommandRawData>>> = None;

pub fn get_subcommand_registry() -> RefMut<'static, Vec<CommandRawData>> {
    let subcommands = unsafe {
        SUBCOMMANDS.get_or_insert_with(|| {
            RefCell::new(Vec::new())
        })
    };

    subcommands.borrow_mut()
}
