use crate::{Context, NodeStatus, TreeNode};

pub trait DecoratorNode {
    fn new(node: Box<dyn TreeNode>) -> Self
    where
        Self: Sized;
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

pub struct ForceSuccess {
    handle: DecoratorNodeHandle,
}

impl TreeNode for ForceSuccess {
    fn tick(&mut self, ctx: &mut Context) -> NodeStatus {
        match self.handle.node.tick(ctx) {
            NodeStatus::Running => NodeStatus::Running,
            _ => NodeStatus::Success,
        }
    }
}

impl DecoratorNode for ForceSuccess {
    fn new(node: Box<dyn TreeNode>) -> Self
    where
        Self: Sized,
    {
        Self {
            handle: DecoratorNodeHandle::new(node),
        }
    }
}

pub struct Repeat {
    count: usize,
    handle: DecoratorNodeHandle,
}

impl Repeat {
    pub fn new(count: usize, node: Box<dyn TreeNode>) -> Self {
        Self {
            count,
            handle: DecoratorNodeHandle::new(node),
        }
    }
}

impl TreeNode for Repeat {
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
