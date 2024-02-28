use std::{any::Any, collections::HashMap, future::Future, str::FromStr};

use node::{
    action::ActionWrapper, composite::CompositeWrapper, decorator::DecoratorWrapper, is_ref_key,
    DataProxy,
};
use parking_lot::RwLock;
use serde::Serialize;
use serde_json::{json, Value};
use thiserror::Error;

pub mod factory;
pub mod node;
pub mod parser;

type Result<T> = std::result::Result<T, BtError>;

#[derive(Error, Debug)]
pub enum BtError {
    #[error("xml parse meet failure")]
    QuickXml(#[from] quick_xml::Error),
    #[error("xml parse meet attr failure")]
    XmlAttr(#[from] quick_xml::events::attributes::AttrError),
    #[error("str parse error")]
    Str(#[from] std::str::Utf8Error),
    #[error("raw error {0}")]
    Raw(String),
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Copy)]
pub enum NodeStatus {
    #[default]
    Idle,
    Success,
    Failure,
    Running,
}

impl NodeStatus {
    pub fn is_completed(&self) -> bool {
        self == &NodeStatus::Success || self == &NodeStatus::Failure
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum NodeType {
    Composite,
    Decorator,
    Action,
}

pub enum NodeWrapper {
    Composite(CompositeWrapper),
    Decorator(DecoratorWrapper),
    Action(ActionWrapper),
}

pub struct TreeNodeWrapper {
    pub node_wrapper: NodeWrapper,
}

impl TreeNodeWrapper {
    pub fn status(&self) -> NodeStatus {
        self.data_proxy_ref().status()
    }

    pub fn reset_status(&mut self) {
        self.data_proxy_ref_mut().reset_status();
    }

    pub fn new(node_wrapper: NodeWrapper) -> Self {
        Self { node_wrapper }
    }

    pub fn node_type(&self) -> NodeType {
        match self.node_wrapper {
            NodeWrapper::Composite(_) => NodeType::Composite,
            NodeWrapper::Decorator(_) => NodeType::Decorator,
            NodeWrapper::Action(_) => NodeType::Action,
        }
    }

    pub fn data_proxy_ref(&self) -> &DataProxy {
        match &self.node_wrapper {
            NodeWrapper::Composite(cp) => &cp.data_proxy,
            NodeWrapper::Decorator(dr) => &dr.data_proxy,
            NodeWrapper::Action(at) => &at.data_proxy,
        }
    }

    pub fn data_proxy_ref_mut(&mut self) -> &mut DataProxy {
        match &mut self.node_wrapper {
            NodeWrapper::Composite(cp) => &mut cp.data_proxy,
            NodeWrapper::Decorator(dr) => &mut dr.data_proxy,
            NodeWrapper::Action(at) => &mut at.data_proxy,
        }
    }

    pub fn uid(&self) -> u16 {
        self.data_proxy_ref().uid()
    }

    pub fn set_uid(&mut self, uid: u16) {
        self.data_proxy_ref_mut().set_uid(uid);
    }

    pub fn node_info(&self) -> String {
        let a = match &self.node_wrapper {
            NodeWrapper::Composite(cp) => cp.debug_info(),
            NodeWrapper::Decorator(dr) => dr.debug_info(),
            NodeWrapper::Action(tn) => tn.debug_info(),
        };

        format!(
            "uid= {} path= {} {a}",
            self.uid(),
            self.data_proxy_ref().path()
        )
    }
}

impl TreeNode for TreeNodeWrapper {
    fn tick(&mut self) -> NodeStatus {
        let uid = self.uid();

        match &mut self.node_wrapper {
            NodeWrapper::Composite(cp) => cp.tick(),
            NodeWrapper::Decorator(dn) => dn.tick(),
            NodeWrapper::Action(tn) => {
                tracing::trace!("action tick: uid= {uid}");
                tn.tick()
            }
        }
    }
}

pub trait TreeNode: Any + Send {
    fn tick(&mut self) -> NodeStatus;
    fn debug_info(&self) -> String {
        format!("Action {}", std::any::type_name::<Self>())
    }
    fn halt(&mut self) {}
}
