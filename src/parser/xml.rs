use std::{
    collections::{HashMap, VecDeque},
    ops::Range,
    sync::{
        atomic::{AtomicU16, Ordering},
        Arc,
    },
};

use crate::{
    factory::Factory,
    node::{Blackboard, DataProxy},
    BtError, NodeWrapper, Result, TreeNode, TreeNodeWrapper,
};
use parking_lot::RwLock;
use quick_xml::{
    events::{attributes::Attributes, Event},
    Reader,
};

struct AttributesWrapper<'a> {
    attrs: Attributes<'a>,
}

impl<'a> AttributesWrapper<'a> {
    fn new(attrs: Attributes<'a>) -> Self {
        Self { attrs }
    }
}

impl<'a> AttributesWrapper<'a> {
    fn get_key(&'a self, key: &str) -> Result<Option<String>> {
        for att in self.attrs.clone() {
            let att = att?;

            if att.key.as_ref() == key.as_bytes() {
                let s = std::str::from_utf8(att.value.as_ref())?.to_string();
                return Ok(Some(s));
            }
        }

        Ok(None)
    }

    fn kv(&self) -> Result<HashMap<String, String>> {
        let mut map = HashMap::new();

        for att in self.attrs.clone() {
            let att = att?;

            let key = std::str::from_utf8(att.key.as_ref())?.to_string();
            let value = std::str::from_utf8(att.value.as_ref())?.to_string();

            map.insert(key, value);
        }

        Ok(map)
    }
}

// only the action nodes leaf nodes
fn create_tree_node_recursively(
    factory: &Factory,
    s: &str,
    check_range: Range<usize>,
    tree_ranges: &HashMap<String, Range<usize>>,
    bb: Arc<RwLock<Blackboard>>,
    uid_generator: &AtomicU16,
) -> Result<Option<TreeNodeWrapper>> {
    tracing::trace!("input: root_str= {s} check_range= {check_range:?}");
    let mut reader = Reader::from_str(&s[check_range]);

    let mut control_nodes = VecDeque::new();

    loop {
        let event = reader.read_event();
        tracing::trace!("event: {event:?}");

        match event {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let name = e.name();
                let element_name = std::str::from_utf8(name.as_ref())?;

                let wrapper = AttributesWrapper::new(e.attributes());

                if factory.composite_types().contains(element_name) {
                    tracing::trace!("composite node");

                    let data_proxy = DataProxy::new(bb.clone());
                    let Some(mut node) =
                        factory.build_composite(element_name, data_proxy, wrapper.kv()?)
                    else {
                        tracing::warn!("can't create node: element_name= {element_name}");
                        continue;
                    };

                    let uid = uid_generator.fetch_add(1, Ordering::SeqCst);
                    node.data_proxy.set_uid(uid);

                    control_nodes.push_front((node, uid));
                } else if factory.decorator_types().contains(element_name) {
                    tracing::trace!("decorator node");

                    let e = reader.read_event()?;
                    tracing::trace!("event after decorator: {e:?}");

                    match reader.read_event()? {
                        Event::Start(e) | Event::Empty(e) => {
                            let node_name = e.name();
                            let node_element_name = std::str::from_utf8(node_name.as_ref())?;
                            let node_wrapper = AttributesWrapper::new(e.attributes());

                            let data_proxy = DataProxy::new(bb.clone());
                            let Some(node) = factory.build_action(
                                node_element_name,
                                data_proxy,
                                node_wrapper.kv()?,
                            ) else {
                                tracing::warn!("can't create node: element_name= {element_name}");

                                continue;
                            };

                            tracing::trace!("has node: {node_element_name}");

                            let data_proxy = DataProxy::new(bb.clone());
                            let Some(mut decorator_node) = factory.build_decorator(
                                element_name,
                                data_proxy,
                                wrapper.kv()?,
                                node,
                            ) else {
                                tracing::warn!("can't create node: element_name= {element_name}");

                                continue;
                            };

                            let decorator_uid = uid_generator.fetch_add(1, Ordering::SeqCst);
                            decorator_node.data_proxy.set_uid(decorator_uid);

                            let inner_node_uid = uid_generator.fetch_add(1, Ordering::SeqCst);
                            decorator_node.inner_node.set_uid(inner_node_uid);

                            let mut node =
                                TreeNodeWrapper::new(NodeWrapper::Decorator(decorator_node));

                            if let Some((control_node, _)) = control_nodes.front_mut() {
                                control_node.add_child(node);
                            } else {
                                return Ok(Some(node));
                            }
                        }
                        _ => {}
                    }
                } else if element_name == "SubTree" {
                    tracing::trace!("SubTree");

                    let wrapper = AttributesWrapper::new(e.attributes());
                    let ref_tree_id = wrapper
                        .get_key("ID")?
                        .ok_or_else(|| BtError::Raw("no ID found for SubTree".to_string()))?;

                    tracing::trace!("SubTree ID: {} tree_ranges= {tree_ranges:?}", ref_tree_id);

                    let range = tree_ranges[&ref_tree_id].clone();

                    let mut subtree_bb = Blackboard::new_with_parent(&bb);

                    let node = create_tree_node_recursively(
                        factory,
                        s,
                        range,
                        tree_ranges,
                        Arc::new(RwLock::new(subtree_bb)),
                        uid_generator,
                    )?
                    .ok_or_else(|| BtError::Raw("no subtree node created".to_string()))?;

                    if let Some((control_node, _)) = control_nodes.front_mut() {
                        control_node.add_child(node);
                    } else {
                        return Ok(Some(node));
                    }
                } else {
                    tracing::trace!("leaf node: {element_name}");

                    let data_proxy = DataProxy::new(bb.clone());
                    let Some(mut node) =
                        factory.build_action(element_name, data_proxy, wrapper.kv()?)
                    else {
                        tracing::warn!("can't create node: element_name= {element_name}");

                        continue;
                    };

                    let uid = uid_generator.fetch_add(1, Ordering::SeqCst);
                    node.set_uid(uid);

                    if let Some((control_node, _)) = control_nodes.front_mut() {
                        control_node.add_child(node);
                    } else {
                        return Ok(Some(node));
                    }
                }
            }
            Ok(Event::End(e)) => {
                let name = e.name();
                let element_name = std::str::from_utf8(name.as_ref())?;

                if factory.composite_types().contains(element_name) {
                    if let Some((control_node, uid)) = control_nodes.pop_front() {
                        let mut control_node_wrapper =
                            TreeNodeWrapper::new(NodeWrapper::Composite(control_node));

                        if let Some((parent_control_node, _)) = control_nodes.front_mut() {
                            parent_control_node.add_child(control_node_wrapper);
                        } else {
                            return Ok(Some(control_node_wrapper));
                        }
                    } else {
                        tracing::warn!("unexpected end: {element_name}");
                    }
                }
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
    }

    Ok(None)
}

pub fn create_bt_tree_from_xml_str(factory: &Factory, s: &str) -> Result<Option<TreeNodeWrapper>> {
    let mut reader = Reader::from_str(s);
    reader.trim_text(true);

    let (main_tree_id, root_range) = loop {
        match reader.read_event() {
            Ok(Event::Start(e)) if e.name().as_ref() == b"root" => {
                let wrapper = AttributesWrapper::new(e.attributes());
                let main_tree_id = wrapper.get_key("main_tree_to_execute")?;

                let end = e.to_end().to_owned();

                let trees_range = reader.read_to_end(end.name())?;
                break (main_tree_id, trees_range);
            }
            Ok(Event::Eof) => {
                return Err(crate::BtError::Raw("no root range found".to_string()));
            }
            _ => {}
        }
    };

    let s = &s[root_range];
    let mut reader = Reader::from_str(s);
    let mut tree_ranges = HashMap::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) if e.name().as_ref() == b"BehaviorTree" => {
                let wrapper = AttributesWrapper::new(e.attributes());

                let Some(id) = wrapper.get_key("ID")? else {
                    return Err(crate::BtError::Raw(
                        "no ID found in BehaviorTree element".to_string(),
                    ));
                };

                let tree_range = reader.read_to_end(e.to_end().to_owned().name())?;

                tree_ranges.insert(id, tree_range);
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
    }

    let main_tree = if let Some(main_tree_id) = main_tree_id {
        tree_ranges.remove(&main_tree_id)
    } else {
        tree_ranges.drain().next().map(|a| a.1)
    };

    let Some(main_tree_range) = main_tree else {
        return Err(BtError::Raw("no main bt tree found".to_string()));
    };

    let bb = Blackboard::default();

    let node = create_tree_node_recursively(
        factory,
        s,
        main_tree_range,
        &tree_ranges,
        Arc::new(RwLock::new(bb)),
        &AtomicU16::new(0),
    )?;

    Ok(node)
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use crate::{factory::boxify_action, node::action::ActionNodeImpl, NodeStatus};

    use super::*;

    fn assets_dir() -> PathBuf {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("assets");

        d
    }

    struct PrintBody;

    impl ActionNodeImpl for PrintBody {
        fn tick_status(&mut self, data_proxy: &mut DataProxy) -> NodeStatus {
            println!("body tick");
            NodeStatus::Success
        }
    }

    struct PrintArm;

    impl ActionNodeImpl for PrintArm {
        fn tick_status(&mut self, data_proxy: &mut DataProxy) -> NodeStatus {
            println!("arm tick");
            NodeStatus::Success
        }
    }

    const XML: &str = r#"
    <?xml version="1.0" encoding="UTF-8"?>
    <root BTCPP_format="4" main_tree_to_execute="main">
        <BehaviorTree ID="main">
            <Sequence>
                <PrintBody body="body"/>
                <PrintArm arm="left_arm"/>
                <PrintArm arm="{arm}"/>
                <Sequence>
                    <PrintBody body="body"/>
                    <PrintArm arm="arm"/>
                    <Sequence>
                        <PrintArm arm="{arm}"/>
                        <ForceSuccess>
                            <PrintBody body="{body}"/>
                        </ForceSuccess>
                        <SubTree ID="bbb"/>
                    </Sequence>
                </Sequence>
            </Sequence>
        </BehaviorTree>
        <BehaviorTree ID="aaa">
            <PrintBody body="body"/>
        </BehaviorTree>
        <BehaviorTree ID="bbb">
            <Sequence>
                <SubTree ID="aaa"/>
                <PrintArm arm="arm"/>
                <SubTree ID="aaa"/>
            </Sequence>
        </BehaviorTree>
    </root>"#;

    #[test]
    fn test_parse() {
        tracing_subscriber::fmt::init();
        let mut factory = Factory::default();
        factory.register_action_node_type(
            "PrintBody".try_into().unwrap(),
            boxify_action(|_, _| Ok(PrintBody)),
        );
        factory.register_action_node_type(
            "PrintArm".try_into().unwrap(),
            boxify_action(|_, _| Ok(PrintArm)),
        );

        let mut xml_path = assets_dir();
        xml_path.push("full.xml");

        let xml_str = std::fs::read_to_string(xml_path).unwrap();

        let node = create_bt_tree_from_xml_str(&factory, &xml_str).unwrap();

        tracing::info!("node: {}", node.is_some());

        if let Some(mut node) = node {
            tracing::info!("node debug info: {}", node.node_info());

            if let NodeWrapper::Composite(cp) = &node.node_wrapper {
                tracing::info!("composite note");
                // tracing::info!("has control node: name= {}", control_node.debug_info());
            }

            loop {
                let res = node.tick();

                if res != NodeStatus::Running {
                    break;
                }
            }
        }
    }
}
