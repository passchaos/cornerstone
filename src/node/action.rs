use serde_json::json;

use crate::{node::strip_ref_tag, NodeStatus, TreeNode, TreeNodeWrapper};

use super::DataProxy;

pub trait ActionNodeImpl: Send {
    fn tick_status(&mut self, data_proxy: &mut DataProxy) -> NodeStatus;

    fn node_info(&self) -> String {
        format!("{}", std::any::type_name::<Self>())
    }
}

pub struct ActionWrapper {
    pub data_proxy: DataProxy,
    node: Box<dyn ActionNodeImpl>,
}

impl TreeNode for ActionWrapper {
    fn tick(&mut self) -> NodeStatus {
        self.node.tick_status(&mut self.data_proxy)
    }

    fn debug_info(&self) -> String {
        format!("Action {}", self.node.node_info())
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

        tracing::debug!("set value for key: key= {} value= {}", output_key, value);

        data_proxy.blackboard().set(output_key, json!(value));

        NodeStatus::Success
    }
}
