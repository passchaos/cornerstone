use std::{any::Any, collections::HashMap, sync::Arc};

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

pub trait ControlNode {
    fn add_child(&mut self, node: Box<dyn TreeNode>);
}

pub trait DecoratorNode {
    fn new(node: Box<dyn TreeNode>) -> Self
    where
        Self: Sized;
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

#[derive(Default)]
pub struct SequenceNode {
    current_child_idx: usize,
    handle: ControlNodeHandle,
}

impl ControlNode for SequenceNode {
    fn add_child(&mut self, node: Box<dyn TreeNode>) {
        self.handle.add_child(node);
    }
}

impl TreeNode for SequenceNode {
    fn tick(&mut self, ctx: &mut Context) -> NodeStatus {
        let from = self.current_child_idx;

        for node in self.handle.child_nodes.iter_mut().skip(from) {
            match node.tick(ctx) {
                NodeStatus::Failure => {
                    return NodeStatus::Failure;
                }
                NodeStatus::Running => {
                    return NodeStatus::Running;
                }
                NodeStatus::Success => (),
            }
        }

        NodeStatus::Success
    }
}

#[derive(Default)]
pub struct ControlNodeHandle {
    child_nodes: Vec<Box<dyn TreeNode>>,
}

impl ControlNode for ControlNodeHandle {
    fn add_child(&mut self, node: Box<dyn TreeNode>) {
        self.child_nodes.push(node);
    }
}

pub struct DecoratorNodeHandle {
    node: Box<dyn TreeNode>,
}

impl DecoratorNode for DecoratorNodeHandle {
    fn new(node: Box<dyn TreeNode>) -> Self
    where
        Self: Sized,
    {
        Self { node }
    }
}

pub struct ForceSuccessNode {
    handle: DecoratorNodeHandle,
}

impl TreeNode for ForceSuccessNode {
    fn tick(&mut self, ctx: &mut Context) -> NodeStatus {
        match self.handle.node.tick(ctx) {
            NodeStatus::Running => NodeStatus::Running,
            _ => NodeStatus::Success,
        }
    }
}

impl DecoratorNode for ForceSuccessNode {
    fn new(node: Box<dyn TreeNode>) -> Self
    where
        Self: Sized,
    {
        Self {
            handle: DecoratorNodeHandle::new(node),
        }
    }
}

pub struct RepeatNode {
    count: usize,
    handle: DecoratorNodeHandle,
}

impl RepeatNode {
    pub fn new(count: usize, node: Box<dyn TreeNode>) -> Self {
        Self {
            count,
            handle: DecoratorNodeHandle::new(node)
        }
    }
}

impl TreeNode for RepeatNode {
    fn tick(&mut self, ctx: &mut Context) -> NodeStatus {
        if self.count == 0 {
            return NodeStatus::Success;
        }

        match self.handle.node.tick(ctx) {
            NodeStatus::Success => {
                self.count -= 1;
                return NodeStatus::Running;
            }
            res => return res,
        }
    }
}
