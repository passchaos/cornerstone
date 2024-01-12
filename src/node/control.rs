use std::collections::HashSet;

use crate::{Context, NodeStatus, TreeNode};

pub trait ControlNode {
    fn add_child(&mut self, node: Box<dyn TreeNode>);
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

pub struct ParallelNode {
    success_threshold: Option<usize>,
    failure_threshold: Option<usize>,
    success_count: usize,
    failure_count: usize,
    completed_list: HashSet<usize>,
    handle: ControlNodeHandle,
}

impl ParallelNode {
    pub fn new(success_threshold: Option<usize>, failure_threshold: Option<usize>) -> Self {
        Self {
            success_threshold,
            failure_threshold,
            success_count: 0,
            failure_count: 0,
            completed_list: HashSet::new(),
            handle: ControlNodeHandle::default(),
        }
    }
}

impl ControlNode for ParallelNode {
    fn add_child(&mut self, node: Box<dyn TreeNode>) {
        self.handle.add_child(node);
    }
}

impl TreeNode for ParallelNode {
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
}
