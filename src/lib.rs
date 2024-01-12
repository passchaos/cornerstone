use std::{any::Any, collections::HashMap, sync::Arc};

pub mod node;

#[derive(Default)]
pub struct Context {
    storage: HashMap<String, Arc<dyn Any>>,
}

impl Context {
    pub fn set<T: 'static>(&mut self, key: String, val: T) {
        self.storage.insert(key, Arc::new(val));
    }

    fn get<T: 'static>(&self, key: &str) -> Option<&T> {
        self.storage.get(key).and_then(|val| val.downcast_ref())
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum NodeStatus {
    Success,
    Failure,
    Running,
}

pub trait TreeNode {
    fn tick(&mut self, ctx: &mut Context) -> NodeStatus;
}

#[derive(Debug, Clone)]
pub struct DataProxy {
    ports_mapping: HashMap<String, String>,
}

impl DataProxy {
    pub fn new(map: HashMap<String, String>) -> Self {
        Self { ports_mapping: map }
    }

    fn ports_mapping_key(&self, key: &str) -> Option<&str> {
        self.ports_mapping.get(key).map(|a| a.as_str())
    }

    pub fn get<'a, T: 'static>(&'a self, ctx: &'a Context, key: &str) -> Option<&T> {
        let key = self.ports_mapping_key(key).unwrap_or(key);

        ctx.get(key)
    }
}
