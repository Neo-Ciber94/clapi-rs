use syn::{Item, ItemMod, UseTree};
use std::path::{Path, PathBuf};
use syn::export::ToTokens;
use syn::visit::Visit;
use std::fmt::{Display, Formatter};

/// Represents the path of a `syn::Item` like: `utils::get_values`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ItemPath {
    path: Vec<String>
}

impl ItemPath {
    pub fn new(name: String) -> Self {
        ItemPath { path: vec![name] }
    }

    pub fn from_path(path: Vec<String>) -> Self {
        assert!(path.len() > 0);
        ItemPath { path }
    }

    pub fn name(&self) -> &str {
        self.path.last()
            .map(|s| s.as_str())
            .unwrap()
    }

    pub fn path(&self) -> &[String]{
        self.path.as_slice()
    }
}

impl Display for ItemPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.join("::"))
    }
}

/// Get all the `syn::Item` that match the given predicate.
pub fn find_items<F>(file_path: &Path, recursive: bool, f: F) -> Vec<(ItemPath, Item)>
    where F: Fn(&Item) -> bool {
    find_items_internal(file_path, recursive, &f)
}

fn find_items_internal<F>(file_path: &Path, recursive: bool, f: &F) -> Vec<(ItemPath, Item)>
    where F: Fn(&Item) -> bool {

    let src = std::fs::read_to_string(&file_path).unwrap();
    let file = syn::parse_file(&src).unwrap();

    // A visitor over all the items
    let mut visitor = ItemVisitor {
        items: vec![],
        mods: vec![],
        path: vec![],
        recursive,
        f
    };

    // Begin
    visitor.visit_file(&file);

    if recursive {
        // Iterate through the items of the all the declared modules
        // This ignore if the mod is public or private
        for item_mod in visitor.mods {
            if let Some(mod_path) = find_item_mod_path(file_path, &item_mod) {
                visitor.items.extend(find_items_internal(&mod_path, true, visitor.f));
            }
        }
    }

    visitor.items
}

fn find_item_mod_path(mod_path: &Path, item_mod: &ItemMod) -> Option<PathBuf> {
    assert!(item_mod.content.is_none(), "expected `mod m` but was `mod m { ... }`");
    let mod_name = item_mod.ident.to_string();
    find_item_mod_path_internal(mod_path, &mod_name)
}

fn find_item_mod_path_internal(mod_path: &Path, mod_name: &str) -> Option<PathBuf> {
    let mut current_path = mod_path;

    // Quick path
    if current_path.is_file() {
        if current_path.file_name().unwrap() == mod_name {
            return Some(mod_path.to_path_buf());
        }
    }

    while let Some(parent) = current_path.parent() {
        let read_dir = parent.read_dir().expect("unable to read directory: `{}`");

        for entry in read_dir {
            if let Ok(dir_entry) = entry {
                let file_path = dir_entry.path();
                if file_path.is_file() {
                    // Ignore no `.rs` files
                    if let Some(ext) = file_path.extension() {
                        if ext != "rs" {
                            continue;
                        }
                    }

                    // The file name
                    let file_name = file_path.file_stem().unwrap();

                    // If have the same name, we found it
                    if file_name == mod_name {
                        return Some(file_path);
                    }
                } else {
                    // If is a file we find the `mod.rs` where the modules are declared
                    let path_name = file_path.file_name().unwrap();
                    if path_name == mod_name {
                        let read_dir = std::fs::read_dir(&file_path).unwrap();
                        for entry in read_dir {
                            if let Ok(dir_entry) = entry {
                                if dir_entry.file_name() == "mod.rs" {
                                    return Some(dir_entry.path());
                                }
                            } else {
                                unreachable!("failed to read `DirEntry`")
                            }
                        }
                    }
                }
            } else {
                unreachable!("failed to read `DirEntry`")
            }
        }

        // We go recursively
        current_path = parent;
    }

    None
}

fn get_item_name(item: &Item) -> Option<String> {
    match item {
        Item::Const(item_const) => {
            Some(item_const.ident.to_string())
        }
        Item::Enum(item_enum) => {
            Some(item_enum.ident.to_string())
        }
        Item::ExternCrate(item_extern_crate) => {
            Some(item_extern_crate.ident.to_string())
        }
        Item::Fn(item_fn) => {
            Some(item_fn.sig.ident.to_string())
        }
        Item::Macro(item_macro) => {
            item_macro.ident.as_ref().map(|ident| ident.to_string())
        }
        Item::Macro2(item_macro2) => {
            Some(item_macro2.ident.to_string())
        }
        Item::Mod(item_mod) => {
            Some(item_mod.ident.to_string())
        }
        Item::Static(item_static) => {
            Some(item_static.ident.to_string())
        }
        Item::Struct(item_struct) => {
            Some(item_struct.ident.to_string())
        }
        Item::Trait(item_trait) => {
            Some(item_trait.ident.to_string())
        }
        Item::TraitAlias(item_trait_alias) => {
            Some(item_trait_alias.ident.to_string())
        }
        Item::Type(item_type) => {
            Some(item_type.ident.to_string())
        }
        Item::Union(item_union) => {
            Some(item_union.ident.to_string())
        }
        Item::Use(item_use) => {
            match &item_use.tree {
                UseTree::Path(use_path) => {
                    // fix: We ignore the UseTree
                    Some(use_path.ident.to_string())
                }
                UseTree::Name(use_name) => {
                    Some(use_name.ident.to_string())
                }
                _ => None
            }
        }
        Item::ForeignMod(_) => {
            None
        }
        Item::Impl(_) => {
            None
        }
        _ => None
    }
}

struct ItemVisitor<'ast, F> {
    items: Vec<(ItemPath, Item)>,
    mods: Vec<&'ast ItemMod>,
    path: Vec<String>,
    recursive: bool,
    f: &'ast F
}

impl<'ast, F> Visit<'ast> for ItemVisitor<'ast, F> where F : Fn(&'ast Item) -> bool {
    fn visit_item(&mut self, i: &'ast Item) {
        if (self.f)(i) {
            let mut path = self.path.clone();
            let item_name = get_item_name(i)
                .unwrap_or_else(|| panic!(
                    "unable to get the ident name of the `Item`: `{}`",
                    i.to_token_stream().to_string())
                );

            path.push(item_name);
            self.items.push((ItemPath::from_path(path), i.clone()));
        }

        if let Item::Mod(item_mod) = i {
            self.path.push(item_mod.ident.to_string());
            self.visit_item_mod(item_mod);
            self.path.pop();
        }
    }

    fn visit_item_mod(&mut self, i: &'ast ItemMod) {
        if let Some((_, content)) = &i.content {
            for item in content {
                self.visit_item(item);
            }
        } else {
            // We only store the mods, later we can visit them
            if self.recursive {
                self.mods.push(i);
            }
        }
    }
}