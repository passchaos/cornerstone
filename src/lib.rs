#![feature(trait_upcasting)]
use std::{any::Any, collections::HashMap, str::FromStr, sync::Arc};

use node::{composite::CompositeNode, decorator::DecoratorNode};
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
    storage: HashMap<String, Arc<dyn Any>>,
}

impl Context {
    pub fn set<T: 'static>(&mut self, key: String, val: T) {
        self.storage.insert(key, Arc::new(val));
    }

    fn get<T: 'static>(&self, key: &str) -> Option<&T> {
        self.storage.get(key).and_then(|val| val.downcast_ref())
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

pub enum TreeNodeWrapper {
    Composite(Box<dyn CompositeNode>),
    Decorator(Box<dyn DecoratorNode>),
    Action(Box<dyn TreeNode>),
}

impl TreeNodeWrapper {
    pub fn node_type(&self) -> NodeType {
        match self {
            Self::Composite(_) => NodeType::Composite,
            Self::Decorator(_) => NodeType::Decorator,
            Self::Action(_) => NodeType::Action,
        }
    }
}

impl TreeNode for TreeNodeWrapper {
    fn tick(&mut self, ctx: &mut Context) -> NodeStatus {
        match self {
            Self::Composite(cp) => cp.tick(ctx),
            Self::Decorator(dn) => dn.tick(ctx),
            Self::Action(tn) => tn.tick(ctx),
        }
    }
}

pub trait TreeNode: Any {
    fn tick(&mut self, ctx: &mut Context) -> NodeStatus;
    fn node_type(&self) -> NodeType {
        NodeType::Action
    }

    fn debug_info(&self) -> String {
        format!(
            "{:?} {}",
            self.node_type(),
            std::any::type_name_of_val(self)
        )
    }
}

pub enum ProxyValue {
    Real(Box<dyn Any>),
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
                    (k, ProxyValue::Real(Box::new(v)))
                }
            })
            .collect();

        Self { ports_mapping: map }
    }

    pub fn insert(&mut self, key: String, value: ProxyValue) {
        self.ports_mapping.insert(key, value);
    }

    pub fn get<'a, T: 'static>(&'a self, ctx: &'a Context, key: &str) -> Option<&T> {
        match self.ports_mapping.get(key) {
            Some(v) => match v {
                ProxyValue::Real(v) => v.downcast_ref(),
                ProxyValue::Ref(r) => ctx.get(r.as_str()),
            },
            None => ctx.get(key),
        }
    }

    pub fn get_string_parsed<'a, T: FromStr>(&'a self, ctx: &'a Context, key: &str) -> Option<T> {
        let a: Option<&String> = self.get(ctx, key);

        a.and_then(|a| a.parse::<T>().ok())
    }
}
