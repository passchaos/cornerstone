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

    pub fn path(&self) -> &str {
        self.data_proxy_ref().path()
    }

    pub fn node_info(&self) -> String {
        let mut info = String::new();

        self.apply_recursive_visitor(&mut |node, layer| {
            info.push_str("\n");

            for _ in 0..layer {
                info.push_str("\t");
            }

            info.push_str(&format!(
                "uid= {} path= {}",
                node.uid(),
                node.data_proxy_ref().full_path()
            ));
        });

        info
    }

    pub fn dot_info(&self) -> String {
        let mut dot_s = String::new();

        dot_s.push_str("digraph G {");

        Self::dot_info_construct(&mut dot_s, self, self);

        dot_s.push_str("}");

        dot_s
    }

    fn dot_info_construct(content: &mut String, node: &TreeNodeWrapper, parent: &TreeNodeWrapper) {
        let p = format!("\"{}_{}\"", parent.uid(), parent.path());

        let node_s = format!("\"{}_{}\"", node.uid(), node.path());

        if p != node_s {
            content.push_str(&format!("{} -> {};\n", p, node_s));
        }

        match &node.node_wrapper {
            NodeWrapper::Action(at) => {}
            NodeWrapper::Composite(cp) => {
                for child_node in &cp.child_nodes {
                    Self::dot_info_construct(content, child_node, node);
                }
            }
            NodeWrapper::Decorator(dr) => {
                Self::dot_info_construct(content, &dr.inner_node, node);
            }
        }
    }

    fn apply_recursive_visitor_impl(&self, layer: u16, visitor: &mut impl FnMut(&Self, u16)) {
        visitor(self, layer);

        match &self.node_wrapper {
            NodeWrapper::Composite(cp) => {
                for child in &cp.child_nodes {
                    child.apply_recursive_visitor_impl(layer + 1, visitor);
                }
            }
            NodeWrapper::Decorator(dn) => {
                dn.inner_node
                    .apply_recursive_visitor_impl(layer + 1, visitor);
            }
            _ => {}
        }
    }

    pub fn apply_recursive_visitor(&self, visitor: &mut impl FnMut(&Self, u16)) {
        self.apply_recursive_visitor_impl(0, visitor);
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

    fn halt(&mut self) {
        match &mut self.node_wrapper {
            NodeWrapper::Composite(cp) => cp.halt(),
            NodeWrapper::Decorator(dn) => dn.halt(),
            NodeWrapper::Action(tn) => {
                tn.halt();
            }
        }
    }
}

pub trait TreeNode: Any + Send {
    fn tick(&mut self) -> NodeStatus;
    fn halt(&mut self) {}
}
