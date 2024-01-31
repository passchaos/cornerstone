use std::collections::{HashMap, HashSet};

use crate::{Context, DataProxy, NodeStatus, TreeNode, TreeNodeWrapper};

trait CompositeNodeImpl {
    fn tick_status(
        &mut self,
        ctx: &mut Context,
        data_proxy: &mut DataProxy,
        child_nodes: &mut Vec<TreeNodeWrapper>,
    ) -> NodeStatus;
}

pub struct CompositeWrapper {
    data_proxy: DataProxy,
    node_wrapper: Box<dyn CompositeNodeImpl>,
    child_nodes: Vec<TreeNodeWrapper>,
}

impl CompositeWrapper {
    pub fn add_child(&mut self, node: TreeNodeWrapper) {
        self.child_nodes.push(node);
    }
}

impl TreeNode for CompositeWrapper {
    fn tick(&mut self, ctx: &mut Context) -> NodeStatus {
        self.node_wrapper
            .tick_status(ctx, &mut self.data_proxy, &mut self.child_nodes)
    }
}

struct SequenceImpl {
    current_child_idx: usize,
}

impl CompositeNodeImpl for SequenceImpl {
    fn tick_status(
        &mut self,
        ctx: &mut Context,
        data_proxy: &mut DataProxy,
        child_nodes: &mut Vec<TreeNodeWrapper>,
    ) -> NodeStatus {
        NodeStatus::Success
    }
}

pub trait Composite {
    fn add_child(&mut self, node: Box<dyn TreeNode>);
}

#[derive(Default)]
struct CompositeHandle {
    data_proxy: DataProxy,
    child_nodes: Vec<Box<dyn TreeNode>>,
}

impl Composite for CompositeHandle {
    fn add_child(&mut self, node: Box<dyn TreeNode>) {
        self.child_nodes.push(node);
    }
}

pub trait CompositeNode: TreeNode + Composite {}

impl<T> CompositeNode for T where T: TreeNode + Composite {}

#[derive(Default)]
pub struct Sequence {
    current_child_idx: usize,
    handle: CompositeHandle,
}

impl Composite for Sequence {
    fn add_child(&mut self, node: Box<dyn TreeNode>) {
        self.handle.add_child(node);
    }
}

impl TreeNode for Sequence {
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
                NodeStatus::Success => {
                    self.current_child_idx += 1;
                }
            }
        }

        NodeStatus::Success
    }

    fn node_type(&self) -> crate::NodeType {
        crate::NodeType::Composite
    }

    fn debug_info(&self) -> String {
        let mut s = format!(
            "Self: {:?} {} | ",
            self.node_type(),
            std::any::type_name_of_val(self)
        );

        for child in &self.handle.child_nodes {
            s.push_str(&format!("\n\t----->child= {}", child.debug_info()));
        }

        s
    }
}

#[derive(Default)]
pub struct Parallel {
    success_threshold: Option<usize>,
    failure_threshold: Option<usize>,
    success_count: usize,
    failure_count: usize,
    completed_list: HashSet<usize>,
    handle: CompositeHandle,
}

pub const PARALLEL_SUCCESS_COUNT: &str = "success_count";
pub const PARALLEL_FAILURE_COUNT: &str = "failure_count";

impl Parallel {
    pub fn new(ports_mapping: HashMap<String, String>) -> Self {
        let data_proxy = DataProxy::new(ports_mapping);
        let handle = CompositeHandle {
            data_proxy,
            ..Default::default()
        };

        Self {
            success_threshold: None,
            failure_threshold: None,
            success_count: 0,
            failure_count: 0,
            completed_list: HashSet::new(),
            handle,
        }
    }
}

impl Composite for Parallel {
    fn add_child(&mut self, node: Box<dyn TreeNode>) {
        self.handle.add_child(node);
    }
}

impl TreeNode for Parallel {
    fn tick(&mut self, ctx: &mut Context) -> NodeStatus {
        let children_count = self.handle.child_nodes.len();

        let success_threshold = self
            .handle
            .data_proxy
            .get_string_parsed::<usize>(ctx, PARALLEL_SUCCESS_COUNT)
            .unwrap_or(self.success_threshold.unwrap_or(children_count));

        let failure_threshold = self
            .handle
            .data_proxy
            .get_string_parsed::<usize>(ctx, PARALLEL_FAILURE_COUNT)
            .unwrap_or(self.failure_threshold.unwrap_or(children_count));

        if children_count == 0 {
            return NodeStatus::Failure;
        }

        for i in 0..children_count {
            if self.completed_list.contains(&i) {
                continue;
            }

            let node = &mut self.handle.child_nodes[i];

            match node.tick(ctx) {
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

    fn node_type(&self) -> crate::NodeType {
        crate::NodeType::Composite
    }

    fn debug_info(&self) -> String {
        let mut s = format!(
            "Self: {:?} {} | ",
            self.node_type(),
            std::any::type_name_of_val(self)
        );

        for child in &self.handle.child_nodes {
            s.push_str(&format!("\n\t----->child= {}", child.debug_info()));
        }

        s
    }
}

#[derive(Default)]
pub struct Selector {
    handle: CompositeHandle,
}

impl Composite for Selector {
    fn add_child(&mut self, node: Box<dyn TreeNode>) {
        self.handle.add_child(node);
    }
}

impl TreeNode for Selector {
    fn tick(&mut self, ctx: &mut Context) -> NodeStatus {
        for node in self.handle.child_nodes.iter_mut() {
            match node.tick(ctx) {
                NodeStatus::Success => return NodeStatus::Success,
                NodeStatus::Running => return NodeStatus::Running,
                NodeStatus::Failure => (),
            }
        }

        NodeStatus::Failure
    }

    fn node_type(&self) -> crate::NodeType {
        crate::NodeType::Composite
    }

    fn debug_info(&self) -> String {
        let mut s = format!(
            "Self: {:?} {} | ",
            self.node_type(),
            std::any::type_name_of_val(self)
        );

        for child in &self.handle.child_nodes {
            s.push_str(&format!("\n\t----->child= {}", child.debug_info()));
        }

        s
    }
}
