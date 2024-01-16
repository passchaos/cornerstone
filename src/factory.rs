use std::collections::{HashMap, HashSet};

use crate::{
    node::{
        composite::{ControlNode, Selector, Sequence},
        decorator::{DecoratorNode, Repeat},
    },
    TreeNode,
};

pub struct Factory {
    composite_types: HashMap<String, Box<dyn Fn() -> Box<dyn ControlNode>>>,
    decorator_types: HashMap<String, Box<dyn Fn() -> Box<dyn DecoratorNode>>>,
    action_node_types: HashMap<String, Box<dyn Fn() -> Box<dyn TreeNode>>>,
}

fn boxify_composite<T, F>(cons: F) -> Box<dyn Fn() -> Box<dyn ControlNode>>
where
    F: 'static + Fn() -> T,
    T: 'static + ControlNode,
{
    Box::new(move || Box::new(cons()))
}

fn boxify_decorator<T, F>(cons: F) -> Box<dyn Fn() -> Box<dyn DecoratorNode>>
where
    F: 'static + Fn() -> T,
    T: 'static + DecoratorNode,
{
    Box::new(move || Box::new(cons()))
}

impl Factory {
    fn register_composite_type(
        &mut self,
        type_name: String,
        constructor: Box<dyn Fn() -> Box<dyn ControlNode>>,
    ) {
        self.composite_types.insert(type_name, constructor);
    }

    fn register_decorator_type(
        &mut self,
        type_name: String,
        constructor: Box<dyn Fn() -> Box<dyn DecoratorNode>>,
    ) {
        self.decorator_types.insert(type_name, constructor);
    }

    pub fn register_action_node_type(
        &mut self,
        type_name: String,
        constructor: Box<dyn Fn() -> Box<dyn TreeNode>>,
    ) {
        self.action_node_types.insert(type_name, constructor);
    }
    fn build_composite(&self, type_name: &str) -> Option<Box<dyn ControlNode>> {
        self.composite_types.get(type_name).map(|c| c())
    }

    fn build_decorator(&self, type_name: &str) -> Option<Box<dyn DecoratorNode>> {
        self.decorator_types.get(type_name).map(|c| c())
    }

    pub fn build_action(&self, type_name: &str) -> Option<Box<dyn TreeNode>> {
        self.action_node_types.get(type_name).map(|c| c())
    }
}

impl Default for Factory {
    fn default() -> Self {
        let mut fac = Self {
            composite_types: HashMap::new(),
            decorator_types: HashMap::new(),
            action_node_types: HashMap::new(),
        };

        fac.register_composite_type("Sequence".to_string(), boxify_composite(Sequence::default));
        fac.register_composite_type("Fallback".to_string(), boxify_composite(Selector::default));

        // fac.register_decorator_type("Repeat".to_string(), boxify_decorator(Repeat::))

        fac
    }
}

pub fn composite_node_types() -> HashSet<&'static str> {
    let mut set = HashSet::new();
    set.insert("Sequnce");
    set.insert("Fallback");

    set
}

pub fn decorator_node_types() -> HashSet<&'static str> {
    let mut set = HashSet::new();
    set.insert("Repeat");
    set.insert("ForceSuccess");

    set
}
