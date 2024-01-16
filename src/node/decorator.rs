use core::num;

use minidom::Node;

use crate::{Context, DataProxy, NodeStatus, TreeNode};

pub trait DecoratorNode {
    fn new(data_proxy: DataProxy, node: Box<dyn TreeNode>) -> Self
    where
        Self: Sized;
}

pub struct DecoratorNodeHandle {
    data_proxy: DataProxy,
    node: Box<dyn TreeNode>,
}

impl DecoratorNode for DecoratorNodeHandle {
    fn new(data_proxy: DataProxy, node: Box<dyn TreeNode>) -> Self
    where
        Self: Sized,
    {
        Self { data_proxy, node }
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
    fn new(data_proxy: DataProxy, node: Box<dyn TreeNode>) -> Self
    where
        Self: Sized,
    {
        Self {
            handle: DecoratorNodeHandle::new(data_proxy, node),
        }
    }
}

pub struct Repeat {
    repeat_count: usize,
    num_cycles: usize,
    handle: DecoratorNodeHandle,
}

impl Repeat {
    pub fn new(count: usize, data_proxy: DataProxy, node: Box<dyn TreeNode>) -> Self {
        Self {
            repeat_count: 0,
            num_cycles: count,
            handle: DecoratorNodeHandle::new(data_proxy, node),
        }
    }
}

pub const NUM_CYCLES: &str = "num_cycles";

impl TreeNode for Repeat {
    fn tick(&mut self, ctx: &mut Context) -> NodeStatus {
        let ports_value = self.handle.data_proxy.get::<String>(ctx, NUM_CYCLES);
        let num_cycles = ports_value
            .and_then(|a| a.parse::<usize>().ok())
            .unwrap_or(self.num_cycles);

        if num_cycles == 0 {
            return NodeStatus::Success;
        }

        match self.handle.node.tick(ctx) {
            NodeStatus::Success | NodeStatus::Failure => {
                self.repeat_count += 1;

                if self.repeat_count == num_cycles {
                    return NodeStatus::Success;
                } else {
                    return NodeStatus::Running;
                }
            }
            res => return res,
        }
    }
}
