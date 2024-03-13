use serde_json::json;

use crate::{NodeStatus, TreeNode};

use super::DataProxy;

pub trait ActionNodeImpl: Send + Sync {
    fn tick_status(&mut self, data_proxy: &mut DataProxy) -> NodeStatus;

    fn node_info(&self) -> String {
        format!("{}", std::any::type_name::<Self>())
    }

    fn halt(&mut self) {}
}

pub struct ActionWrapper {
    pub data_proxy: DataProxy,
    node: Box<dyn ActionNodeImpl>,
}

impl TreeNode for ActionWrapper {
    fn tick(&mut self) -> NodeStatus {
        if self.data_proxy.status() == NodeStatus::Idle {
            self.data_proxy.set_status(NodeStatus::Running);
        }

        let new_status = self.node.tick_status(&mut self.data_proxy);
        self.data_proxy.set_status(new_status);

        new_status
    }

    fn halt(&mut self) {
        tracing::debug!("halt action: {}", std::any::type_name::<Self>());

        self.node.halt();
    }
}

impl ActionWrapper {
    pub fn new(data_proxy: DataProxy, node: Box<dyn ActionNodeImpl>) -> Self {
        Self { data_proxy, node }
    }
}

#[derive(Default)]
pub struct SetBlackboard;

impl ActionNodeImpl for SetBlackboard {
    fn tick_status(&mut self, data_proxy: &mut DataProxy) -> NodeStatus {
        let Some(output_key) = data_proxy.get_input::<String>("output_key") else {
            return NodeStatus::Failure;
        };

        let Some(value) = data_proxy.get_input::<String>("value") else {
            return NodeStatus::Failure;
        };

        data_proxy.blackboard().set(output_key, json!(value));

        NodeStatus::Success
    }
}
