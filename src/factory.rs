use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
    sync::Arc,
};

use regex::Regex;
use tracing_subscriber::registry::Data;

use crate::{
    node::{
        action::ActionWrapper,
        composite::{CompositeNodeImpl, CompositeWrapper, Parallel, Selector, Sequence},
        decorator::{
            DecoratorNodeImpl, DecoratorWrapper, ForceFailure, ForceSuccess, Inverter, Repeat,
            Retry,
        },
    },
    BtError, NodeWrapper, TreeNode, TreeNodeWrapper,
};
use crate::{
    node::{Blackboard, DataProxy},
    Result,
};

pub struct Factory {
    composite_tcs: HashMap<String, Box<dyn Fn(DataProxy, Attrs) -> CompositeWrapper>>,
    decorator_tcs:
        HashMap<String, Box<dyn Fn(DataProxy, Attrs, TreeNodeWrapper) -> DecoratorWrapper>>,
    action_node_tcs:
        HashMap<ActionRegex, Box<dyn Fn(&str, Attrs) -> OuterResult<Box<dyn TreeNode>>>>,
}

type Attrs = HashMap<String, String>;

fn boxify_composite<T, F>(cons: F) -> Box<dyn Fn(DataProxy, Attrs) -> CompositeWrapper>
where
    F: 'static + Fn(&Attrs) -> T,
    T: 'static + CompositeNodeImpl + Send,
{
    Box::new(move |data_proxy, attrs| {
        let node_wrapper = Box::new(cons(&attrs));

        CompositeWrapper::new(data_proxy, node_wrapper)
    })
}

fn boxify_decorator<T, F>(
    cons: F,
) -> Box<dyn Fn(DataProxy, Attrs, TreeNodeWrapper) -> DecoratorWrapper>
where
    F: 'static + Fn(&Attrs) -> T,
    T: 'static + DecoratorNodeImpl,
{
    Box::new(move |data_proxy, attrs, inner_node| {
        let node_wrapper = Box::new(cons(&attrs));
        DecoratorWrapper::new(data_proxy, node_wrapper, inner_node)
    })
}

#[derive(Clone, Debug)]
pub struct ActionRegex {
    regex: Regex,
}

impl TryFrom<&str> for ActionRegex {
    type Error = BtError;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let regex = Regex::new(value)
            .map_err(|e| BtError::Raw(format!("convert to regex meet failure: err= {e}")))?;

        Ok(Self { regex })
    }
}

impl From<Regex> for ActionRegex {
    fn from(value: Regex) -> Self {
        Self { regex: value }
    }
}

impl Deref for ActionRegex {
    type Target = Regex;

    fn deref(&self) -> &Self::Target {
        &self.regex
    }
}

impl PartialEq for ActionRegex {
    fn eq(&self, other: &Self) -> bool {
        self.regex.as_str() == other.regex.as_str()
    }
}

impl Eq for ActionRegex {}

impl std::hash::Hash for ActionRegex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.regex.as_str().hash(state)
    }
}

type OuterError = Box<dyn std::error::Error + Send + Sync>;
type OuterResult<T> = std::result::Result<T, OuterError>;

pub fn boxify_action<T, F>(cons: F) -> Box<dyn Fn(&str, Attrs) -> OuterResult<Box<dyn TreeNode>>>
where
    F: 'static + Fn(&str, Attrs) -> OuterResult<T>,
    T: 'static + TreeNode,
{
    Box::new(move |type_name, attrs| {
        let res = cons(type_name, attrs)?;
        Ok(Box::new(res))
    })
}

impl Factory {
    pub fn composite_types(&self) -> HashSet<&str> {
        self.composite_tcs.keys().map(|a| a.as_str()).collect()
    }

    pub fn decorator_types(&self) -> HashSet<&str> {
        self.decorator_tcs.keys().map(|a| a.as_str()).collect()
    }

    fn register_composite_type(
        &mut self,
        type_name: String,
        constructor: Box<dyn Fn(DataProxy, Attrs) -> CompositeWrapper>,
    ) {
        self.composite_tcs.insert(type_name, constructor);
    }

    fn register_decorator_type(
        &mut self,
        type_name: String,
        constructor: Box<dyn Fn(DataProxy, Attrs, TreeNodeWrapper) -> DecoratorWrapper>,
    ) {
        self.decorator_tcs.insert(type_name, constructor);
    }

    pub fn register_action_node_type(
        &mut self,
        type_name_pat: ActionRegex,
        constructor: Box<dyn Fn(&str, Attrs) -> OuterResult<Box<dyn TreeNode>>>,
    ) {
        self.action_node_tcs.insert(type_name_pat, constructor);
    }
    pub fn build_composite(
        &self,
        type_name: &str,
        data_proxy: DataProxy,
        attrs: Attrs,
    ) -> Option<CompositeWrapper> {
        self.composite_tcs
            .get(type_name)
            .map(|c| c(data_proxy, attrs))
    }

    pub fn build_decorator(
        &self,
        type_name: &str,
        data_proxy: DataProxy,
        attrs: Attrs,
        node: TreeNodeWrapper,
    ) -> Option<DecoratorWrapper> {
        self.decorator_tcs
            .get(type_name)
            .map(|c| c(data_proxy, attrs, node))
    }

    pub fn build_action(
        &self,
        type_name: &str,
        data_proxy: DataProxy,
        attrs: Attrs,
    ) -> Option<TreeNodeWrapper> {
        for (type_regex, constructor) in &self.action_node_tcs {
            if type_regex.is_match(type_name) {
                let node = match constructor(type_name, attrs.clone()) {
                    Ok(n) => n,
                    Err(e) => {
                        tracing::error!("run action builder meet failure: err= {e}");
                        continue;
                    }
                };

                let action_wrapper = ActionWrapper::new(data_proxy, node);
                return Some(TreeNodeWrapper::new(NodeWrapper::Action(action_wrapper)));
            } else {
                continue;
            }
        }

        None
        //    self.action_node_tcs
        //         .get(type_name)
        //         .map(|c| c(attrs))
        //         .map(|a| TreeNodeWrapper::new(NodeWrapper::Action(a)))
    }
}

impl Default for Factory {
    fn default() -> Self {
        let mut fac = Self {
            composite_tcs: HashMap::new(),
            decorator_tcs: HashMap::new(),
            action_node_tcs: HashMap::new(),
        };

        fac.register_composite_type(
            "Sequence".to_string(),
            boxify_composite(|_| Sequence::default()),
        );
        fac.register_composite_type(
            "Fallback".to_string(),
            boxify_composite(|_| Selector::default()),
        );
        fac.register_composite_type(
            "Parallel".to_string(),
            boxify_composite(|_| Parallel::default()),
        );

        fac.register_decorator_type(
            "ForceSuccess".to_string(),
            boxify_decorator(|_| ForceSuccess::default()),
        );
        fac.register_decorator_type(
            "ForceFailure".to_string(),
            boxify_decorator(|_| ForceFailure::default()),
        );
        fac.register_decorator_type(
            "Inverter".to_string(),
            boxify_decorator(|_| Inverter::default()),
        );
        fac.register_decorator_type(
            "Repeat".to_string(),
            boxify_decorator(|_| Repeat::default()),
        );
        fac.register_decorator_type(
            "RetryUntilSuccessful".to_string(),
            boxify_decorator(|_| Retry::default()),
        );

        fac
    }
}
