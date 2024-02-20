use std::{any::Any, collections::HashMap, str::FromStr};

use node::{composite::CompositeWrapper, decorator::DecoratorWrapper, is_ref_key};
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

#[derive(Default)]
pub struct Context {
    storage: RwLock<HashMap<String, Value>>,
}

impl Context {
    pub fn set<T: Serialize>(&mut self, key: String, val: T) {
        self.storage.write().insert(key, json!(val));
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        let key = if is_ref_key(key) {
            let ref_key = key.replace("{", "").replace("}", "");
            ref_key
        } else {
            key.to_string()
        };

        let guard = self.storage.read();
        guard.get(&key).cloned()
    }
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
    Action(Box<dyn TreeNode>),
}

pub struct TreeNodeWrapper {
    pub uid: u16,
    pub node_wrapper: NodeWrapper,
}

impl TreeNodeWrapper {
    pub fn new(node_wrapper: NodeWrapper) -> Self {
        Self {
            uid: 0,
            node_wrapper,
        }
    }

    pub fn node_type(&self) -> NodeType {
        match self.node_wrapper {
            NodeWrapper::Composite(_) => NodeType::Composite,
            NodeWrapper::Decorator(_) => NodeType::Decorator,
            NodeWrapper::Action(_) => NodeType::Action,
        }
    }

    pub fn node_info(&self) -> String {
        let a = match &self.node_wrapper {
            NodeWrapper::Composite(cp) => cp.node_info(),
            NodeWrapper::Decorator(dr) => dr.node_info(),
            NodeWrapper::Action(tn) => tn.debug_info(),
        };

        format!("uid= {} {a}", self.uid)
    }
}

impl TreeNode for TreeNodeWrapper {
    fn tick(&mut self, ctx: &mut Context) -> NodeStatus {
        match &mut self.node_wrapper {
            NodeWrapper::Composite(cp) => cp.tick(ctx),
            NodeWrapper::Decorator(dn) => dn.tick(ctx),
            NodeWrapper::Action(tn) => {
                tracing::trace!("action tick: uid= {}", self.uid);
                tn.tick(ctx)
            }
        }
    }
}

pub trait TreeNode: Any + Send {
    fn tick(&mut self, ctx: &mut Context) -> NodeStatus;
    fn debug_info(&self) -> String {
        format!("Action {}", std::any::type_name::<Self>())
    }
}
