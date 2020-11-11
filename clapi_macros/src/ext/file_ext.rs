use syn::{File, ItemFn, Item, ItemMod};
use crate::ItemModExt;

pub trait FileExt {
    fn contains_fn(&self, item_fn: &ItemFn, include_modules: bool) -> bool;
    fn find_fn_module(&self, item_fn: &ItemFn) -> Option<ItemMod>;
    fn get_fns(&self) -> Vec<ItemFn>;
}

impl FileExt for File {
    fn contains_fn(&self, item_fn: &ItemFn, include_modules: bool) -> bool {
        for item in &self.items {
            match item {
                Item::Fn(inner_fn) => if item_fn == inner_fn {
                    return true;
                },
                Item::Mod(inner_mod) if include_modules && inner_mod.contains_fn(item_fn) => {
                    return true;
                },
                _ => {}
            }
        }

        false
    }

    fn find_fn_module(&self, item_fn: &ItemFn) -> Option<ItemMod> {
        self.items.iter().find_map(|item| {
            match item {
                Item::Mod(inner_mod) if inner_mod.contains_fn(item_fn) => {
                    Some(inner_mod.clone())
                },
                _ => None,
            }
        })
    }

    fn get_fns(&self) -> Vec<ItemFn> {
        let mut ret = Vec::new();
        for item in &self.items {
            match item {
                Item::Fn(item_fn) => { ret.push(item_fn.clone())},
                _ => {}
            }
        }

        ret
    }
}