use std::collections::HashSet;

use crate::{Context, DataProxy, NodeStatus, TreeNode};

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
                NodeStatus::Success => (),
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
    pub fn new(success_threshold: Option<usize>, failure_threshold: Option<usize>) -> Self {
        Self {
            success_threshold,
            failure_threshold,
            success_count: 0,
            failure_count: 0,
            completed_list: HashSet::new(),
            handle: CompositeHandle::default(),
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

            if self.success_count >= self.success_threshold.unwrap_or(children_count) {
                return NodeStatus::Success;
            }

            if self.failure_count >= self.failure_threshold.unwrap_or(children_count) {
                return NodeStatus::Failure;
            }
        }

        NodeStatus::Running
    }

    fn node_type(&self) -> crate::NodeType {
        crate::NodeType::Composite
    }

    fn debug_info(&self) -> String {
        let mut s = format!("Self: {:?} | ", self.node_type());

        for child in &self.handle.child_nodes {
            s.push_str(&format!("child= {:?} | ", child.node_type()));
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
        let mut s = format!("Self: {:?} | ", self.node_type());

        for child in &self.handle.child_nodes {
            s.push_str(&format!("child= {:?} | ", child.debug_info()));
        }

        s
    }
}
