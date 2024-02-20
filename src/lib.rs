use std::{any::Any, collections::HashMap, str::FromStr};

use node::{
    action::ActionWrapper, composite::CompositeWrapper, decorator::DecoratorWrapper, is_ref_key,
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

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum NodeStatus {
    Success,
    Failure,
    Running,
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

    pub fn uid(&self) -> u16 {
        match &self.node_wrapper {
            NodeWrapper::Composite(cp) => cp.data_proxy.uid(),
            NodeWrapper::Decorator(dr) => dr.data_proxy.uid(),
            NodeWrapper::Action(at) => at.data_proxy.uid(),
        }
    }

    pub fn set_uid(&mut self, uid: u16) {
        match &mut self.node_wrapper {
            NodeWrapper::Composite(cp) => {
                cp.data_proxy.set_uid(uid);
            }
            NodeWrapper::Decorator(dr) => {
                dr.data_proxy.set_uid(uid);
            }
            NodeWrapper::Action(at) => {
                at.data_proxy.set_uid(uid);
            }
        }
    }

    pub fn node_info(&self) -> String {
        let a = match &self.node_wrapper {
            NodeWrapper::Composite(cp) => cp.node_info(),
            NodeWrapper::Decorator(dr) => dr.node_info(),
            NodeWrapper::Action(tn) => tn.debug_info(),
        };

        format!("uid= {} {a}", self.uid())
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
}
