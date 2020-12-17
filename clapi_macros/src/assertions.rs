use syn::{File, Item, ItemFn, Visibility};

pub fn is_top_function(item_fn: &ItemFn, file: &File) {
    let found = file
        .items
        .iter()
        .filter_map(|item| matches_map!(item, Item::Fn(f) => f))
        // We don't compare attribute because we don't know the order they are expanded
        .any(|f| f.sig == item_fn.sig && f.vis == item_fn.vis && f.block == item_fn.block);

    if !found {
        panic!(
            "`{}` is not a top function.\
                \n`command`s must be free functions and be declared outside a module.",
            item_fn.sig.ident
        )
    }
}

pub fn is_public(item_fn: &ItemFn) {
    match item_fn.vis {
        Visibility::Public(_) => {}
        _ => {
            panic!(
                "subcommands must be declared public: `{}` is not public",
                item_fn.sig.ident
            );
        }
    }
}
