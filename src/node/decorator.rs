use crate::{NodeStatus, TreeNode, TreeNodeWrapper};

use super::DataProxy;

pub trait DecoratorNodeImpl: Send + Sync {
    fn tick_status(
        &mut self,
        data_proxy: &mut DataProxy,
        inner_node: &mut TreeNodeWrapper,
    ) -> NodeStatus;
    fn node_info(&self) -> String {
        std::any::type_name::<Self>().to_string()
    }
    fn reset_state(&mut self) {}
}

pub struct DecoratorWrapper {
    pub data_proxy: DataProxy,
    node_wrapper: Box<dyn DecoratorNodeImpl>,
    pub inner_node: Box<TreeNodeWrapper>,
}

impl TreeNode for DecoratorWrapper {
    fn tick(&mut self) -> NodeStatus {
        if self.data_proxy.status() == NodeStatus::Idle {
            self.data_proxy.set_status(NodeStatus::Running);
        }

        let tick_status = self
            .node_wrapper
            .tick_status(&mut self.data_proxy, &mut self.inner_node);
        if tick_status.is_completed() {
            self.halt();
        }

        self.data_proxy.set_status(tick_status);

        tick_status
    }

    fn halt(&mut self) {
        tracing::debug!("halt self: {}", std::any::type_name::<Self>());

        self.node_wrapper.reset_state();
        self.reset_inner();
    }
}

impl DecoratorWrapper {
    pub fn new(
        data_proxy: DataProxy,
        node_wrapper: Box<dyn DecoratorNodeImpl>,
        inner_node: TreeNodeWrapper,
    ) -> Self {
        Self {
            data_proxy,
            node_wrapper,
            inner_node: Box::new(inner_node),
        }
    }

    pub fn reset_inner(&mut self) {
        if self.inner_node.status() == NodeStatus::Running {
            self.inner_node.halt();
        }

        self.inner_node.reset_status();
    }
}

#[derive(Default)]
pub struct ForceSuccess;

impl DecoratorNodeImpl for ForceSuccess {
    fn tick_status(
        &mut self,
        _data_proxy: &mut DataProxy,
        inner_node: &mut TreeNodeWrapper,
    ) -> NodeStatus {
        match inner_node.tick() {
            NodeStatus::Running => NodeStatus::Running,
            _ => NodeStatus::Success,
        }
    }
}

#[derive(Default)]
pub struct ForceFailure;

impl DecoratorNodeImpl for ForceFailure {
    fn tick_status(
        &mut self,
        _data_proxy: &mut DataProxy,
        inner_node: &mut TreeNodeWrapper,
    ) -> NodeStatus {
        match inner_node.tick() {
            NodeStatus::Running => NodeStatus::Running,
            _ => NodeStatus::Failure,
        }
    }
}

#[derive(Default)]
pub struct Inverter;

impl DecoratorNodeImpl for Inverter {
    fn tick_status(
        &mut self,
        _data_proxy: &mut DataProxy,
        inner_node: &mut TreeNodeWrapper,
    ) -> NodeStatus {
        match inner_node.tick() {
            NodeStatus::Running => NodeStatus::Running,
            NodeStatus::Failure => NodeStatus::Success,
            NodeStatus::Success => NodeStatus::Failure,
            NodeStatus::Idle => NodeStatus::Failure,
        }
    }
}

#[derive(Default)]
pub struct Repeat {
    repeat_count: usize,
}

pub const NUM_CYCLES: &str = "num_cycles";

impl DecoratorNodeImpl for Repeat {
    fn tick_status(
        &mut self,
        data_proxy: &mut DataProxy,
        inner_node: &mut TreeNodeWrapper,
    ) -> NodeStatus {
        let num_cycles = data_proxy.get_input(NUM_CYCLES).unwrap_or(1);

        tracing::trace!("bb num cycles: {num_cycles}");

        if num_cycles == 0 {
            return NodeStatus::Success;
        }

        match inner_node.tick() {
            a @ NodeStatus::Success | a @ NodeStatus::Failure => {
                self.repeat_count += 1;

                if self.repeat_count == num_cycles {
                    a
                } else {
                    NodeStatus::Running
                }
            }
            res => res,
        }
    }

    fn reset_state(&mut self) {
        std::mem::swap(self, &mut Self::default());
    }
}

#[derive(Default)]
pub struct Retry {
    try_count: usize,
}

impl DecoratorNodeImpl for Retry {
    fn tick_status(
        &mut self,
        data_proxy: &mut DataProxy,
        inner_node: &mut TreeNodeWrapper,
    ) -> NodeStatus {
        let num_attempts = data_proxy.get_input(NUM_ATTEMPTS).unwrap_or(1);

        while self.try_count <= num_attempts {
            match inner_node.tick() {
                NodeStatus::Idle => return NodeStatus::Failure,
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

    fn reset_state(&mut self) {
        std::mem::swap(self, &mut Self::default());
    }
}

pub const NUM_ATTEMPTS: &str = "num_attempts";

pub struct SubTree {
    _id: String,
}

impl SubTree {
    pub fn new(id: String) -> Self {
        Self { _id: id }
    }
}

impl DecoratorNodeImpl for SubTree {
    fn tick_status(
        &mut self,
        _data_proxy: &mut DataProxy,
        inner_node: &mut TreeNodeWrapper,
    ) -> NodeStatus {
        inner_node.tick()
    }
}
