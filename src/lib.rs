use std::{any::Any, collections::HashMap, str::Utf8Error, sync::Arc};

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
    #[error("dom xml parse meet failure")]
    Minidom(#[from] minidom::Error),
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

pub trait TreeNode {
    fn tick(&mut self, ctx: &mut Context) -> NodeStatus;
}

pub enum ProxyValue {
    Real(Box<dyn Any>),
    Ref(String),
}

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
    pub fn new(map: HashMap<String, ProxyValue>) -> Self {
        Self { ports_mapping: map }
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
}
