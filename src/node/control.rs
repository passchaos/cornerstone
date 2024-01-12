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
