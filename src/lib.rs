use std::{any::Any, collections::HashMap, str::FromStr, sync::Arc};

use node::{composite::CompositeWrapper, decorator::DecoratorWrapper};
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

pub fn is_ref_key(key: &str) -> bool {
    key.starts_with("{") && key.ends_with("}")
}

#[derive(Default)]
pub struct Context {
    storage: HashMap<String, Value>,
}

impl Context {
    pub fn set<T: Serialize>(&mut self, key: String, val: T) {
        self.storage.insert(key, json!(val));
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        let key = if is_ref_key(key) {
            let ref_key = key.replace("{", "").replace("}", "");
            ref_key
        } else {
            key.to_string()
        };

        self.storage.get(&key)
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

pub enum ProxyValue {
    Real(Value),
    Ref(String),
}

#[derive(Default)]
pub struct DataProxy {
    ports_mapping: HashMap<String, ProxyValue>,
}

impl std::fmt::Debug for DataProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataProxy")
            .field("keys", &self.ports_mapping.keys())
            .finish()
    }
}

impl DataProxy {
    pub fn new(map: HashMap<String, String>) -> Self {
        let map = map
            .into_iter()
            .map(|(k, v)| {
                if v.starts_with("{") && v.ends_with("}") {
                    (k, ProxyValue::Ref(v))
                } else {
                    (k, ProxyValue::Real(json!(v)))
                }
            })
            .collect();

        Self { ports_mapping: map }
    }

    pub fn insert(&mut self, key: String, value: ProxyValue) {
        self.ports_mapping.insert(key, value);
    }

    pub fn get<'a>(&'a self, ctx: &'a Context, key: &str) -> Option<&Value> {
        match self.ports_mapping.get(key) {
            Some(v) => match v {
                ProxyValue::Real(v) => Some(v),
                ProxyValue::Ref(r) => ctx.get(r.as_str()),
            },
            None => ctx.get(key),
        }
    }

    pub fn get_string_parsed<'a, T: FromStr>(&'a self, ctx: &'a Context, key: &str) -> Option<T> {
        let Some(a) = self.get(ctx, key) else {
            return None;
        };

        let Some(a) = a.as_str() else {
            tracing::warn!("not a string value");
            return None;
        };

        a.parse::<T>().ok()
    }
}
