use std::collections::HashSet;

use crate::{NodeStatus, TreeNode, TreeNodeWrapper};

use super::DataProxy;

pub trait CompositeNodeImpl: Send + Sync {
    fn tick_status(
        &mut self,
        data_proxy: &mut DataProxy,
        child_nodes: &mut Vec<TreeNodeWrapper>,
    ) -> NodeStatus;
    fn node_info(&self) -> String {
        std::any::type_name::<Self>().to_string()
    }
    fn reset_state(&mut self);
}

pub struct CompositeWrapper {
    pub data_proxy: DataProxy,
    node_wrapper: Box<dyn CompositeNodeImpl>,
    pub child_nodes: Vec<TreeNodeWrapper>,
}

impl CompositeWrapper {
    pub fn new(data_proxy: DataProxy, node_wrapper: Box<dyn CompositeNodeImpl>) -> Self {
        Self {
            data_proxy,
            node_wrapper,
            child_nodes: vec![],
        }
    }

    pub fn add_child(&mut self, node: TreeNodeWrapper) {
        self.child_nodes.push(node);
    }

    pub fn reset_children(&mut self) {
        for child_node in &mut self.child_nodes {
            tracing::trace!(
                "child node status: uid= {} {:?}",
                child_node.uid(),
                child_node.status()
            );

            if child_node.status() == NodeStatus::Running {
                child_node.halt();
            }
            child_node.reset_status();
        }
    }
}

impl TreeNode for CompositeWrapper {
    fn tick(&mut self) -> NodeStatus {
        if self.data_proxy.status() == NodeStatus::Idle {
            self.data_proxy.set_status(NodeStatus::Running);
        }

        let tick_status = self
            .node_wrapper
            .tick_status(&mut self.data_proxy, &mut self.child_nodes);

        if tick_status.is_completed() {
            self.halt();
        }

        self.data_proxy.set_status(tick_status);

        tick_status
    }

    fn halt(&mut self) {
        tracing::debug!("halt self: {}", std::any::type_name::<Self>());
        self.node_wrapper.reset_state();
        self.reset_children();
    }
}

#[derive(Default)]
pub struct Sequence {
    current_child_idx: usize,
}

impl CompositeNodeImpl for Sequence {
    fn tick_status(
        &mut self,
        _data_proxy: &mut DataProxy,
        child_nodes: &mut Vec<TreeNodeWrapper>,
    ) -> NodeStatus {
        let from = self.current_child_idx;

        for node in child_nodes.iter_mut().skip(from) {
            match node.tick() {
                NodeStatus::Failure => {
                    return NodeStatus::Failure;
                }
                NodeStatus::Running => {
                    return NodeStatus::Running;
                }
                NodeStatus::Success => {
                    self.current_child_idx += 1;
                }
                NodeStatus::Idle => return NodeStatus::Failure,
            }
        }

        NodeStatus::Success
    }

    fn node_info(&self) -> String {
        format!("Sequence: current_child_idx= {}", self.current_child_idx)
    }

    fn reset_state(&mut self) {
        self.current_child_idx = 0;
    }
}

#[derive(Default)]
pub struct Parallel {
    success_threshold: Option<usize>,
    failure_threshold: Option<usize>,
    success_count: usize,
    failure_count: usize,
    completed_list: HashSet<usize>,
}

pub const PARALLEL_SUCCESS_COUNT: &str = "success_count";
pub const PARALLEL_FAILURE_COUNT: &str = "failure_count";

impl CompositeNodeImpl for Parallel {
    fn tick_status(
        &mut self,
        data_proxy: &mut DataProxy,
        child_nodes: &mut Vec<TreeNodeWrapper>,
    ) -> NodeStatus {
        let children_count = child_nodes.len();

        let success_threshold = data_proxy
            .get_input(PARALLEL_SUCCESS_COUNT)
            .unwrap_or(self.success_threshold.unwrap_or(children_count));

        let failure_threshold = data_proxy
            .get_input(PARALLEL_FAILURE_COUNT)
            .unwrap_or(self.failure_threshold.unwrap_or(children_count));

        if children_count == 0 {
            return NodeStatus::Failure;
        }

        for (i, node) in child_nodes.iter_mut().enumerate().take(children_count) {
            if self.completed_list.contains(&i) {
                continue;
            }

            match node.tick() {
                NodeStatus::Idle => return NodeStatus::Failure,
                NodeStatus::Failure => {
                    self.failure_count += 1;
                }
                NodeStatus::Success => {
                    self.success_count += 1;
                }
                NodeStatus::Running => continue,
            }

            self.completed_list.insert(i);

            if self.success_count >= success_threshold {
                return NodeStatus::Success;
            }

            if self.failure_count >= failure_threshold {
                return NodeStatus::Failure;
            }
        }

        NodeStatus::Running
    }

    fn reset_state(&mut self) {
        std::mem::swap(self, &mut Self::default());
    }
}

#[derive(Default)]
pub struct Selector {
    current_child_idx: usize,
}

impl CompositeNodeImpl for Selector {
    fn tick_status(
        &mut self,
        _data_proxy: &mut DataProxy,
        child_nodes: &mut Vec<TreeNodeWrapper>,
    ) -> NodeStatus {
        for node in child_nodes.iter_mut().skip(self.current_child_idx) {
            match node.tick() {
                NodeStatus::Idle => return NodeStatus::Failure,
                NodeStatus::Success => {
                    self.reset_state();
                    return NodeStatus::Success;
                }
                NodeStatus::Running => return NodeStatus::Running,
                NodeStatus::Failure => {
                    self.current_child_idx += 1;
                }
            }
        }

        NodeStatus::Failure
    }

    fn reset_state(&mut self) {
        std::mem::swap(self, &mut Self::default());
    }
}
