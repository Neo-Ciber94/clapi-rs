use syn::{ItemFn, ItemMod, Item};

pub trait ItemModExt {
    fn contains_fn(&self, item_fn: &ItemFn) -> bool;
}

impl ItemModExt for ItemMod {
    fn contains_fn(&self, item_fn: &ItemFn) -> bool {
        if let Some((_, content)) = &self.content {
            for item in content {
                match item {
                    Item::Fn(inner_fn) if inner_fn == item_fn => {
                        return true;
                    },
                    Item::Mod(inner_mod) if inner_mod.contains_fn(item_fn) => {
                        return true;
                    },
                    _ => {}
                }
            }
        }

        false
    }
}