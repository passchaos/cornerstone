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

pub struct ForceFailure {
    handle: DecoratorNodeHandle,
}

impl TreeNode for ForceFailure {
    fn tick(&mut self, ctx: &mut Context) -> NodeStatus {
        match self.handle.node.tick(ctx) {
            NodeStatus::Running => NodeStatus::Running,
            _ => NodeStatus::Failure,
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

impl Decorator for ForceFailure {
    fn new(data_proxy: DataProxy, node: Box<dyn TreeNode>) -> Self
    where
        Self: Sized,
    {
        Self {
            handle: DecoratorNodeHandle::new(data_proxy, node),
        }
    }
}

pub struct Inverter {
    handle: DecoratorNodeHandle,
}

impl TreeNode for Inverter {
    fn tick(&mut self, ctx: &mut Context) -> NodeStatus {
        match self.handle.node.tick(ctx) {
            NodeStatus::Running => NodeStatus::Running,
            NodeStatus::Failure => NodeStatus::Success,
            NodeStatus::Success => NodeStatus::Failure,
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

impl Decorator for Inverter {
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
    pub fn new_with_count(count: usize, data_proxy: DataProxy, node: Box<dyn TreeNode>) -> Self {
        Self {
            repeat_count: 0,
            num_cycles: count,
            handle: DecoratorNodeHandle::new(data_proxy, node),
        }
    }
}

impl Decorator for Repeat {
    fn new(data_proxy: DataProxy, node: Box<dyn TreeNode>) -> Self
    where
        Self: Sized,
    {
        Self::new_with_count(0, data_proxy, node)
    }
}

pub const NUM_CYCLES: &str = "num_cycles";

impl TreeNode for Repeat {
    fn tick(&mut self, ctx: &mut Context) -> NodeStatus {
        let num_cycles = self
            .handle
            .data_proxy
            .get_string_parsed::<usize>(ctx, NUM_CYCLES)
            .unwrap_or(self.num_cycles);

        tracing::trace!("bb num cycles: {num_cycles}");

        if num_cycles == 0 {
            return NodeStatus::Success;
        }

        match self.handle.node.tick(ctx) {
            a @ NodeStatus::Success | a @ NodeStatus::Failure => {
                self.repeat_count += 1;

                if self.repeat_count == num_cycles {
                    return a;
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

pub struct Retry {
    try_count: usize,
    handle: DecoratorNodeHandle,
}

impl Decorator for Retry {
    fn new(data_proxy: DataProxy, node: Box<dyn TreeNode>) -> Self
    where
        Self: Sized,
    {
        Self {
            try_count: 0,
            handle: DecoratorNodeHandle::new(data_proxy, node),
        }
    }
}

pub const NUM_ATTEMPTS: &str = "num_attempts";

impl TreeNode for Retry {
    fn tick(&mut self, ctx: &mut Context) -> NodeStatus {
        let num_attempts = self
            .handle
            .data_proxy
            .get_string_parsed::<usize>(ctx, NUM_ATTEMPTS)
            .unwrap_or(1);

        while self.try_count <= num_attempts {
            match self.handle.node.tick(ctx) {
                NodeStatus::Failure => {
                    self.try_count += 1;
                    continue;
                }
                NodeStatus::Running => return NodeStatus::Running,
                NodeStatus::Success => return NodeStatus::Success,
            }
        }

        NodeStatus::Failure
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
