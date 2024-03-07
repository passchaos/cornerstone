use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
    sync::Arc,
};

use regex::Regex;

use crate::{
    node::{
        action::{ActionNodeImpl, ActionWrapper, SetBlackboard},
        composite::{CompositeNodeImpl, CompositeWrapper, Parallel, Selector, Sequence},
        decorator::{
            DecoratorNodeImpl, DecoratorWrapper, ForceFailure, ForceSuccess, Inverter, Repeat,
            Retry, SubTree,
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
    decorator_tcs: HashMap<
        String,
        Box<dyn Fn(DataProxy, Attrs, TreeNodeWrapper) -> OuterResult<DecoratorWrapper>>,
    >,
    action_node_tcs:
        HashMap<ActionRegex, Box<dyn Fn(&str, DataProxy, Attrs) -> OuterResult<ActionWrapper>>>,
}

type Attrs = HashMap<String, String>;
type OuterError = Box<dyn std::error::Error + Send + Sync>;
type OuterResult<T> = std::result::Result<T, OuterError>;

fn boxify_composite<T, F>(cons: F) -> Box<dyn Fn(DataProxy, Attrs) -> CompositeWrapper>
where
    F: 'static + Fn(&Attrs) -> T,
    T: 'static + CompositeNodeImpl,
{
    Box::new(move |data_proxy, attrs| {
        let node_wrapper = Box::new(cons(&attrs));

        CompositeWrapper::new(data_proxy, node_wrapper)
    })
}

fn boxify_decorator<T, F>(
    cons: F,
) -> Box<dyn Fn(DataProxy, Attrs, TreeNodeWrapper) -> OuterResult<DecoratorWrapper>>
where
    F: 'static + Fn(&Attrs) -> OuterResult<T>,
    T: 'static + DecoratorNodeImpl,
{
    Box::new(move |data_proxy, attrs, inner_node| {
        let node_wrapper = Box::new(cons(&attrs)?);
        Ok(DecoratorWrapper::new(data_proxy, node_wrapper, inner_node))
    })
}

pub fn boxify_action<T, F>(
    cons: F,
) -> Box<dyn Fn(&str, DataProxy, Attrs) -> OuterResult<ActionWrapper>>
where
    F: 'static + Fn(&str, Attrs) -> OuterResult<T>,
    T: 'static + ActionNodeImpl,
{
    Box::new(move |type_name, data_proxy, attrs| {
        let res = cons(type_name, attrs)?;

        Ok(ActionWrapper::new(data_proxy, Box::new(res)))
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
        constructor: Box<
            dyn Fn(DataProxy, Attrs, TreeNodeWrapper) -> OuterResult<DecoratorWrapper>,
        >,
    ) {
        self.decorator_tcs.insert(type_name, constructor);
    }

    pub fn register_action_node_type(
        &mut self,
        type_name_pat: ActionRegex,
        constructor: Box<dyn Fn(&str, DataProxy, Attrs) -> OuterResult<ActionWrapper>>,
    ) {
        self.action_node_tcs.insert(type_name_pat, constructor);
    }
    pub fn build_composite(
        &self,
        type_name: &str,
        mut data_proxy: DataProxy,
        attrs: Attrs,
    ) -> Option<CompositeWrapper> {
        for (key, value) in attrs.clone() {
            data_proxy.add_input(key, value);
        }

        self.composite_tcs
            .get(type_name)
            .map(|c| c(data_proxy, attrs))
    }

    pub fn build_decorator(
        &self,
        type_name: &str,
        mut data_proxy: DataProxy,
        attrs: Attrs,
        node: TreeNodeWrapper,
    ) -> Option<DecoratorWrapper> {
        for (key, value) in attrs.clone() {
            data_proxy.add_input(key, value);
        }

        self.decorator_tcs
            .get(type_name)
            .and_then(|c| match c(data_proxy, attrs, node) {
                Ok(v) => Some(v),
                Err(e) => {
                    tracing::error!("create {type_name} meet failure: err= {e}");
                    None
                }
            })
    }

    pub fn build_action(
        &self,
        type_name: &str,
        mut data_proxy: DataProxy,
        attrs: Attrs,
    ) -> Option<TreeNodeWrapper> {
        for (key, value) in attrs.clone() {
            data_proxy.add_input(key, value);
        }

        for (type_regex, constructor) in &self.action_node_tcs {
            if type_regex.is_match(type_name) {
                let action_wrapper = match constructor(type_name, data_proxy, attrs.clone()) {
                    Ok(n) => n,
                    Err(e) => {
                        tracing::error!("run action builder meet failure: err= {e}");
                        return None;
                    }
                };

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
            boxify_decorator(|_| Ok(ForceSuccess::default())),
        );
        fac.register_decorator_type(
            "ForceFailure".to_string(),
            boxify_decorator(|_| Ok(ForceFailure::default())),
        );
        fac.register_decorator_type(
            "Inverter".to_string(),
            boxify_decorator(|_| Ok(Inverter::default())),
        );
        fac.register_decorator_type(
            "Repeat".to_string(),
            boxify_decorator(|_| Ok(Repeat::default())),
        );
        fac.register_decorator_type(
            "RetryUntilSuccessful".to_string(),
            boxify_decorator(|_| Ok(Retry::default())),
        );
        fac.register_decorator_type(
            "SubTree".to_string(),
            boxify_decorator(|attrs| {
                let id = attrs
                    .get("ID")
                    .ok_or_else(|| BtError::Raw(format!("no id found in SubTree attributes")))?;

                Ok(SubTree::new(id.to_string()))
            }),
        );

        fac.register_action_node_type(
            "^SetBlackboard$".try_into().unwrap(),
            boxify_action(|_, _| Ok(SetBlackboard::default())),
        );

        fac
    }
}
