use std::collections::HashMap;

use crate::{
    node::control::{Selector, Sequence},
    TreeNode,
};

pub struct Factory {
    node_types: HashMap<String, Box<dyn Fn() -> Box<dyn TreeNode>>>,
}

pub fn boxify<T, F>(cons: F) -> Box<dyn Fn() -> Box<dyn TreeNode>>
where
    F: 'static + Fn() -> T,
    T: 'static + TreeNode,
{
    Box::new(move || Box::new(cons()))
}

impl Factory {
    pub fn register_node_type(
        &mut self,
        type_name: String,
        constructor: Box<dyn Fn() -> Box<dyn TreeNode>>,
    ) {
        self.node_types.insert(type_name, constructor);
    }

    pub fn build(&self, type_name: &str) -> Option<Box<dyn TreeNode>> {
        self.node_types.get(type_name).map(|c| c())
    }
}

impl Default for Factory {
    fn default() -> Self {
        let mut fac = Self {
            node_types: HashMap::new(),
        };

        fac.register_node_type("Sequence".to_string(), boxify(Sequence::default));
        fac.register_node_type("Fallback".to_string(), boxify(Selector::default));

        fac
    }
}
