use syn::{Item, ItemMod, UseTree, File};
use std::path::{Path, PathBuf};
use syn::export::{ToTokens, Formatter};
use syn::visit::Visit;
use crate::utils::NamePath;
use std::fmt::Debug;

// A result item of a query.
pub struct QueryItem<T>{
    pub path: PathBuf,
    pub name_path: NamePath,
    pub file: File,
    pub item: T
}

impl<T: Debug> Debug for QueryItem<T>{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryItem")
            .field("path", &self.path)
            .field("name_path", &self.name_path)
            .field("file", &self.file)
            .field("item", &self.item)
            .finish()
    }
}

impl<T: Clone> Clone for QueryItem<T>{
    fn clone(&self) -> Self {
        QueryItem {
            path: self.path.clone(),
            name_path: self.name_path.clone(),
            file: self.file.clone(),
            item: self.item.clone()
        }
    }
}

pub fn get_mod_path(path: &Path) -> Vec<String> {
    let mut ret_path = Vec::new();

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

        ret_path.push(item.trim_end_matches(".rs").to_string());
    }

    ret_path
}

pub fn find_items<F>(file_path: &Path, visit_mods: bool, recursive: bool, f: F) -> Vec<QueryItem<Item>>
    where F: Fn(&Item) -> bool {
    find_items_internal(file_path, true, visit_mods, recursive, &|item| {
        if f(item){
            Some(item.clone())
        } else {
            None
        }
    })
}

pub fn find_map_items<T, F>(file_path: &Path, visit_mods: bool, recursive: bool, f: F) -> Vec<QueryItem<T>>
    where F: Fn(&Item) -> Option<T> {
    find_items_internal(file_path, true, visit_mods, recursive, &f)
}

fn find_items_internal<T, F>(file_path: &Path, is_root: bool, visit_mods: bool, recursive: bool, f: &F) -> Vec<QueryItem<T>>
    where F: Fn(&Item) -> Option<T> {

    let src = std::fs::read_to_string(&file_path).unwrap();
    let file = syn::parse_file(&src).unwrap();
    let path = if !is_root { get_mod_path(file_path) } else { Vec::new() };

    // A visitor over all the items
    let mut visitor = ItemVisitor {
        items: vec![],
        mods: vec![],
        path,
        visit_mods,
        recursive,
        f
    };

    // Begin
    visitor.visit_file(&file);

    // Deconstruct the visitor
    let ItemVisitor { items, mods, f, .. } = visitor;
    let mut items = items
        .into_iter()
        .map(|(name_path, item)| {
            QueryItem {
                path: file_path.to_path_buf(),
                file: file.clone(),
                name_path,
                item,
            }
        })
        .collect::<Vec<_>>();

    if visit_mods {
        // Iterate through the items of the all the declared modules
        // This ignore if the mod is public or private
        for item_mod in mods {
            if let Some(mod_path) = find_item_mod_path(file_path, &item_mod) {
                items.extend(find_items_internal(&mod_path, false, visit_mods, recursive, f));
            }
        }
    }

    items
}

fn find_item_mod_path(mod_path: &Path, item_mod: &ItemMod) -> Option<PathBuf> {
    assert!(item_mod.content.is_none(), "{}", "expected `mod m` but was `mod m { ... }`");
    let mod_name = item_mod.ident.to_string();
    find_item_mod_path_internal(mod_path, &mod_name)
}

fn find_item_mod_path_internal(mod_path: &Path, mod_name: &str) -> Option<PathBuf> {
    let mut current_path = mod_path;

    // Quick path
    if current_path.is_file() && current_path.file_name().unwrap() == mod_name {
        return Some(mod_path.to_path_buf());
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

struct ItemVisitor<'ast, T, F> {
    items: Vec<(NamePath, T)>,          // Items and its Path
    mods: Vec<&'ast ItemMod>,           // Modules, empty if `visit_mods` is false
    path: Vec<String>,                  // Only for internal use, a buffer for the path
    visit_mods: bool,                   // Indicates if visit the declared modules
    recursive: bool,                    // Indicates if visit the inner modules
    f: &'ast F                          // Predicate function
}

impl<'ast, T, F> Visit<'ast> for ItemVisitor<'ast, T, F> where F : Fn(&'ast Item) -> Option<T> {
    fn visit_item(&mut self, i: &'ast Item) {
        if let Some(item) = (self.f)(i) {
            let mut path = self.path.clone();
            let item_name = get_item_name(i)
                .unwrap_or_else(|| panic!(
                    "unable to get the ident name of the `Item`: `{}`",
                    i.to_token_stream().to_string())
                );

            path.push(item_name);
            self.items.push((NamePath::from_path(path), item));
        }

        if let Item::Mod(item_mod) = i {
            self.path.push(item_mod.ident.to_string());
            self.visit_item_mod(item_mod);
            self.path.pop();
        }
    }

    fn visit_item_mod(&mut self, i: &'ast ItemMod) {
        if let Some((_, content)) = &i.content {
            if self.recursive {
                for item in content {
                    self.visit_item(item);
                }
            }
        } else {
            // We only store the mods, later we can visit them
            if self.visit_mods {
                self.mods.push(i);
            }
        }
    }
}