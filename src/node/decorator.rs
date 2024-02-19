use std::collections::HashMap;

use crate::{Context, DataProxy, NodeStatus, TreeNode, TreeNodeWrapper};

pub trait DecoratorNodeImpl: Send {
    fn tick_status(
        &mut self,
        ctx: &mut Context,
        data_proxy: &mut DataProxy,
        inner_node: &mut TreeNodeWrapper,
    ) -> NodeStatus;
    fn node_info(&self) -> String {
        format!("{}", std::any::type_name::<Self>())
    }
}

pub struct DecoratorWrapper {
    data_proxy: DataProxy,
    node_wrapper: Box<dyn DecoratorNodeImpl>,
    pub inner_node: Box<TreeNodeWrapper>,
}

impl TreeNode for DecoratorWrapper {
    fn tick(&mut self, ctx: &mut Context) -> NodeStatus {
        self.node_wrapper
            .tick_status(ctx, &mut self.data_proxy, &mut self.inner_node)
    }
}

impl DecoratorWrapper {
    pub fn new(
        ports_mapping: HashMap<String, String>,
        node_wrapper: Box<dyn DecoratorNodeImpl>,
        inner_node: TreeNodeWrapper,
    ) -> Self {
        let data_proxy = DataProxy::new(ports_mapping);

        Self {
            data_proxy,
            node_wrapper,
            inner_node: Box::new(inner_node),
        }
    }

    pub fn node_info(&self) -> String {
        let s = format!("Decorator: {}", self.node_wrapper.node_info());

        let inner_node_info = self.inner_node.node_info();

        format!("{s} | inner= {inner_node_info}")
    }
}

#[derive(Default)]
pub struct ForceSuccess;

impl DecoratorNodeImpl for ForceSuccess {
    fn tick_status(
        &mut self,
        ctx: &mut Context,
        data_proxy: &mut DataProxy,
        inner_node: &mut TreeNodeWrapper,
    ) -> NodeStatus {
        match inner_node.tick(ctx) {
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
        ctx: &mut Context,
        data_proxy: &mut DataProxy,
        inner_node: &mut TreeNodeWrapper,
    ) -> NodeStatus {
        match inner_node.tick(ctx) {
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
        ctx: &mut Context,
        data_proxy: &mut DataProxy,
        inner_node: &mut TreeNodeWrapper,
    ) -> NodeStatus {
        match inner_node.tick(ctx) {
            NodeStatus::Running => NodeStatus::Running,
            NodeStatus::Failure => NodeStatus::Success,
            NodeStatus::Success => NodeStatus::Failure,
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
        ctx: &mut Context,
        data_proxy: &mut DataProxy,
        inner_node: &mut TreeNodeWrapper,
    ) -> NodeStatus {
        let num_cycles = data_proxy
            .get_string_parsed::<usize>(ctx, NUM_CYCLES)
            .unwrap_or(1);

        tracing::trace!("bb num cycles: {num_cycles}");

        if num_cycles == 0 {
            return NodeStatus::Success;
        }

        match inner_node.tick(ctx) {
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
}

#[derive(Default)]
pub struct Retry {
    try_count: usize,
}

impl DecoratorNodeImpl for Retry {
    fn tick_status(
        &mut self,
        ctx: &mut Context,
        data_proxy: &mut DataProxy,
        inner_node: &mut TreeNodeWrapper,
    ) -> NodeStatus {
        let num_attempts = data_proxy
            .get_string_parsed::<usize>(ctx, NUM_ATTEMPTS)
            .unwrap_or(1);

        while self.try_count <= num_attempts {
            match inner_node.tick(ctx) {
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
}

pub const NUM_ATTEMPTS: &str = "num_attempts";
