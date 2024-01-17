use core::num;

use minidom::Node;

use crate::{Context, DataProxy, NodeStatus, TreeNode};

pub trait Decorator {
    fn new(data_proxy: DataProxy, node: Box<dyn TreeNode>) -> Self
    where
        Self: Sized;
}

pub struct DecoratorNodeHandle {
    data_proxy: DataProxy,
    node: Box<dyn TreeNode>,
}

impl Decorator for DecoratorNodeHandle {
    fn new(data_proxy: DataProxy, node: Box<dyn TreeNode>) -> Self
    where
        Self: Sized,
    {
        Self { data_proxy, node }
    }
}

pub trait DecoratorNode: TreeNode + Decorator {}

impl<T> DecoratorNode for T where T: TreeNode + Decorator {}

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

    fn node_type(&self) -> crate::NodeType {
        crate::NodeType::Decorator
    }

    fn debug_info(&self) -> String {
        let mut s = format!(
            "Self: {:?} {}",
            self.node_type(),
            std::any::type_name_of_val(self)
        );

        s.push_str(&format!(
            "\n\t=========>child= {}",
            self.handle.node.debug_info()
        ));

        s
    }
}

impl Decorator for ForceSuccess {
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

    fn node_type(&self) -> crate::NodeType {
        crate::NodeType::Decorator
    }
}

pub struct SubTree {
    handle: DecoratorNodeHandle,
}

impl Decorator for SubTree {
    fn new(data_proxy: DataProxy, node: Box<dyn TreeNode>) -> Self
    where
        Self: Sized,
    {
        Self {
            handle: DecoratorNodeHandle::new(data_proxy, node),
        }
    }
}

impl TreeNode for SubTree {
    fn tick(&mut self, ctx: &mut Context) -> NodeStatus {
        self.handle.node.tick(ctx)
    }

    fn node_type(&self) -> crate::NodeType {
        crate::NodeType::Decorator
    }

    fn debug_info(&self) -> String {
        let mut s = format!("SubTree");
        s.push_str(&format!(" child= {:?}", self.handle.node.debug_info()));

        s
    }
}
