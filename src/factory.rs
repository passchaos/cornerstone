use std::collections::{HashMap, HashSet};

use crate::{
    node::{
        composite::{CompositeNodeImpl, CompositeWrapper, Parallel, Selector, Sequence},
        decorator::{
            Decorator, DecoratorNode, DecoratorNodeImpl, DecoratorWrapper, ForceFailure,
            ForceSuccess, ForceSuccessImpl, Inverter, Repeat, Retry,
        },
    },
    DataProxy, NodeWrapper, TreeNode, TreeNodeWrapper,
};

pub struct Factory {
    composite_tcs: HashMap<String, Box<dyn Fn(Attrs) -> CompositeWrapper>>,
    decorator_tcs: HashMap<String, Box<dyn Fn(Attrs, TreeNodeWrapper) -> DecoratorWrapper>>,
    action_node_tcs: HashMap<String, Box<dyn Fn(Attrs) -> Box<dyn TreeNode>>>,
}

type Attrs = HashMap<String, String>;

fn boxify_composite<T, F>(cons: F) -> Box<dyn Fn(Attrs) -> CompositeWrapper>
where
    F: 'static + Fn(&Attrs) -> T,
    T: 'static + CompositeNodeImpl,
{
    Box::new(move |attrs| {
        let node_wrapper = Box::new(cons(&attrs));

        CompositeWrapper::new(attrs, node_wrapper)
    })
}

fn boxify_decorator<T, F>(cons: F) -> Box<dyn Fn(Attrs, TreeNodeWrapper) -> DecoratorWrapper>
where
    F: 'static + Fn(&Attrs) -> T,
    T: 'static + DecoratorNodeImpl,
{
    Box::new(move |attrs, inner_node| {
        let node_wrapper = Box::new(cons(&attrs));
        DecoratorWrapper::new(attrs, node_wrapper, inner_node)
    })
}

pub fn boxify_action<T, F>(cons: F) -> Box<dyn Fn(Attrs) -> Box<dyn TreeNode>>
where
    F: 'static + Fn(Attrs) -> T,
    T: 'static + TreeNode,
{
    Box::new(move |attrs| Box::new(cons(attrs)))
}

impl Factory {
    pub fn composite_types(&self) -> HashSet<&str> {
        self.composite_tcs.keys().map(|a| a.as_str()).collect()
    }

    pub fn decorator_types(&self) -> HashSet<&str> {
        self.decorator_tcs.keys().map(|a| a.as_str()).collect()
    }

    pub fn action_node_types(&self) -> HashSet<&str> {
        self.action_node_tcs.keys().map(|a| a.as_str()).collect()
    }

    fn register_composite_type(
        &mut self,
        type_name: String,
        constructor: Box<dyn Fn(Attrs) -> CompositeWrapper>,
    ) {
        self.composite_tcs.insert(type_name, constructor);
    }

    fn register_decorator_type(
        &mut self,
        type_name: String,
        constructor: Box<dyn Fn(Attrs, TreeNodeWrapper) -> DecoratorWrapper>,
    ) {
        self.decorator_tcs.insert(type_name, constructor);
    }

    pub fn register_action_node_type(
        &mut self,
        type_name: String,
        constructor: Box<dyn Fn(Attrs) -> Box<dyn TreeNode>>,
    ) {
        self.action_node_tcs.insert(type_name, constructor);
    }
    pub fn build_composite(&self, type_name: &str, attrs: Attrs) -> Option<CompositeWrapper> {
        self.composite_tcs.get(type_name).map(|c| c(attrs))
    }

    pub fn build_decorator(
        &self,
        type_name: &str,
        attrs: Attrs,
        node: TreeNodeWrapper,
    ) -> Option<DecoratorWrapper> {
        self.decorator_tcs.get(type_name).map(|c| c(attrs, node))
    }

    pub fn build_action(&self, type_name: &str, attrs: Attrs) -> Option<TreeNodeWrapper> {
        self.action_node_tcs
            .get(type_name)
            .map(|c| c(attrs))
            .map(|a| TreeNodeWrapper::new(NodeWrapper::Action(a)))
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
        // fac.register_composite_type(
        //     "Fallback".to_string(),
        //     boxify_composite(|_| Selector::default()),
        // );
        // fac.register_composite_type(
        //     "Parallel".to_string(),
        //     boxify_composite(|attrs| Parallel::new(attrs)),
        // );

        fac.register_decorator_type(
            "ForceSuccess".to_string(),
            boxify_decorator(|_| ForceSuccessImpl::default()),
        );
        // fac.register_decorator_type(
        //     "ForceFailure".to_string(),
        //     boxify_decorator(|_| ForceFailure::new(DataProxy::default())),
        // );
        // fac.register_decorator_type(
        //     "Inverter".to_string(),
        //     boxify_decorator(|attrs, node| {
        //         let data_proxy = DataProxy::new(attrs);
        //         Inverter::new(data_proxy, node)
        //     }),
        // );
        // fac.register_decorator_type(
        //     "Repeat".to_string(),
        //     boxify_decorator(|attrs, node| {
        //         let data_proxy = DataProxy::new(attrs);
        //         Repeat::new(data_proxy, node)
        //     }),
        // );
        // fac.register_decorator_type(
        //     "RetryUntilSuccessful".to_string(),
        //     boxify_decorator(|attrs, node| {
        //         let data_proxy = DataProxy::new(attrs);

        //         Retry::new(data_proxy, node)
        //     }),
        // );

        fac
    }
}
