use syn::{ItemFn, ItemMod};
use crate::utils::path_to_string;

pub trait AttrQuery<'a>{
    fn contains_attribute(&self, path: &str) -> bool;
    fn drop_attribute(self, path: &str) -> Self;
    fn drop_all_attributes(self, paths: &[&'a str]) -> Self;
}

impl<'a> AttrQuery<'a> for ItemFn {
    fn contains_attribute(&self, path: &str) -> bool {
        self.attrs.iter().any(|att| {
            path_to_string(&att.path) == path
        })
    }

    fn drop_attribute(mut self, path: &str) -> Self{
        self.attrs = self.attrs.iter()
            .filter(|att| path_to_string(&att.path) != path)
            .cloned()
            .collect();

        self
    }

    fn drop_all_attributes(mut self, paths: &[&'a str]) -> Self{
        self.attrs = self.attrs.iter()
            .filter(|att| {
                let att_path = path_to_string(&att.path);
                !paths.contains(&att_path.as_str())
            })
            .cloned()
            .collect();

        self
    }
}

impl<'a> AttrQuery<'a> for ItemMod {
    fn contains_attribute(&self, path: &str) -> bool {
        self.attrs.iter().any(|att| {
            path_to_string(&att.path) == path
        })
    }

    fn drop_attribute(mut self, path: &str) -> Self{
        self.attrs = self.attrs.iter()
            .filter(|att| path_to_string(&att.path) != path)
            .cloned()
            .collect();

        self
    }

    fn drop_all_attributes(mut self, paths: &[&'a str]) -> Self{
        self.attrs = self.attrs.iter()
            .filter(|att| {
                let att_path = path_to_string(&att.path);
                !paths.contains(&att_path.as_str())
            })
            .cloned()
            .collect();

        self
    }
}