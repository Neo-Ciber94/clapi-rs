use syn::{PatType, Path};
use syn::export::ToTokens;

/// Quote the result of an expression
macro_rules! quote_expr {
    ($value:expr) => {{
        let val = &$value;
        quote::quote!(#val)
    }};
}

macro_rules! matches_map {
        ($expression:expr, $pattern:pat => $ret:expr) => {
            match $expression {
                $pattern => Some($ret),
                _ => None,
            }
        };
    }

pub fn pat_type_to_string(pat_type: &PatType) -> String {
    let arg_name = pat_type.pat.to_token_stream().to_string();
    let type_name = pat_type.ty.to_token_stream().into_iter()
        .map(|t| t.to_string())
        .collect::<Vec<String>>()
        .join("");

    format!("{} : {}", arg_name, type_name)
}

pub fn path_to_string(path: &Path) -> String {
    path.segments.iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<String>>()
        .join("::")
}

pub use mod_query::*;
mod mod_query {
    use syn::{ItemMod, ItemFn, Item, File};
    use syn::export::fmt::Display;
    use syn::export::Formatter;
    use syn::visit::Visit;
    use crate::AttrQuery;

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub struct ItemModNode {
        ancestors: Vec<ItemMod>,
        item_mod: ItemMod
    }

    impl ItemModNode {
        pub fn new(ancestors: Vec<ItemMod>, item_mod: ItemMod) -> Self {
            ItemModNode {
                ancestors,
                item_mod
            }
        }

        pub fn root(&self) -> Option<&ItemMod> {
            if self.ancestors.is_empty() {
                None
            } else {
                self.ancestors.get(0)
            }
        }

        pub fn depth(&self) -> usize {
            self.ancestors.len()
        }

        pub fn ancestors(&self) -> &[ItemMod]{
            self.ancestors.as_slice()
        }

        pub fn item_mod(&self) -> &ItemMod {
            &self.item_mod
        }

        pub fn into_vec(self) -> Vec<ItemMod> {
            self.iter_from_root()
                .cloned()
                .collect::<Vec<ItemMod>>()
        }

        pub fn iter_from_root(&self) -> impl Iterator<Item=&'_ ItemMod> {
            let mut iter = self.ancestors.iter().rev();
            let mut item_mod = Some(&self.item_mod);
            std::iter::from_fn(move || {
                if let Some(next) = iter.next().or_else(|| item_mod.take()) {
                    Some(next)
                } else {
                    None
                }
            })
        }
    }

    impl Display for ItemModNode {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let s = self.iter_from_root()
                .map(|s| s.ident.to_string())
                .collect::<Vec<String>>()
                .join("::");

            write!(f, "{}", s)
        }
    }

    struct FindItemFnVisitor<'ast>{
        marker: &'ast str,
        item_fn: Option<&'ast ItemFn>,
        mod_list: Vec<ItemMod>,
        found: bool,
    }

    impl<'ast> FindItemFnVisitor<'ast> {
        pub fn new(marker: &'ast str) -> Self {
            FindItemFnVisitor {
                marker,
                item_fn: None,
                mod_list: vec![],
                found: false
            }
        }
    }

    impl<'ast> Visit<'ast> for FindItemFnVisitor<'ast> {
        fn visit_item_fn(&mut self, item_fn: &'ast ItemFn) {
            if self.found {
                return;
            }

            if item_fn.contains_attribute(self.marker) {
                self.item_fn = Some(item_fn);
                self.found = true;
            }
        }

        fn visit_item_mod(&mut self, item_mod: &'ast ItemMod) {
            if self.found {
                return;
            }

            if let Some((_, content)) = item_mod.content.as_ref() {
                self.mod_list.push(item_mod.clone());

                for item in content {
                    match item {
                        Item::Fn(inner_fn) => self.visit_item_fn(inner_fn),
                        Item::Mod(inner_mod) => {
                            self.visit_item_mod(inner_mod);
                        }
                        _ => {}
                    }
                }

                if !self.found {
                    self.mod_list.pop();
                }
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct ItemFnAndModNode {
        pub item_fn: ItemFn,
        pub item_mod_node: Option<ItemModNode>
    }

    pub fn find_item_fn(file: &File, marker: &str) -> Option<ItemFnAndModNode>{
        let mut visitor = FindItemFnVisitor::new(marker);
        visitor.visit_file(file);

        if visitor.found {
            let FindItemFnVisitor { item_fn, mut mod_list, .. } = visitor;

            let item_fn = item_fn.unwrap().clone();
            let item_mod_node = if mod_list.is_empty() {
                None
            } else {
                if mod_list.len() == 1 {
                    Some(ItemModNode {
                        ancestors: Vec::new(),
                        item_mod: mod_list.swap_remove(0)
                    })
                } else {
                    Some(ItemModNode {
                        ancestors: mod_list[..mod_list.len() - 1].to_vec(),
                        item_mod: mod_list.pop().unwrap()
                    })
                }
            };

            Some(ItemFnAndModNode { item_fn, item_mod_node })
        } else {
            None
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use quote::*;

        #[test]
        fn find_item_fn_test1() {
            let tokens = quote! {
                mod a {}
                mod b {
                    mod c {
                        #[marker]
                        fn test(){}
                    }
                }
            };

            let file = syn::parse2::<File>(tokens).unwrap();
            let result = find_item_fn(&file, "marker").unwrap();

            assert_eq!(result.item_mod_node.unwrap().to_string(), "b::c");
            assert_eq!(result.item_fn.sig.ident.to_string(), "test")
        }

        #[test]
        fn find_item_fn_test2() {
            let tokens = quote! {
                 #[marker]
                 fn test(){}
            };

            let file = syn::parse2::<File>(tokens).unwrap();
            let result = find_item_fn(&file, "marker").unwrap();

            assert!(result.item_mod_node.is_none());
            assert_eq!(result.item_fn.sig.ident.to_string(), "test")
        }
    }
}