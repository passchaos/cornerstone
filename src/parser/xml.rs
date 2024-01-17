use std::{
    any::Any,
    collections::{HashMap, VecDeque},
    fs::read,
    hash::Hash,
    ops::Range,
    str::FromStr,
};

use crate::{
    factory::{self, Factory},
    node::composite::{Composite, CompositeNode},
    Context, NodeStatus, NodeType, Result, TreeNode,
};
use quick_xml::{
    events::{attributes::Attributes, BytesStart, Event},
    name::{self, QName},
    Reader,
};

fn handle_byte_start<'a>(e: BytesStart<'a>) {
    let name = e.name();
    let attributes = e.attributes();

    for att in attributes {
        println!("name= {name:?} att= {att:?}");
    }
}

pub fn from_str_dom(s: &str) -> Result<()> {
    let e = minidom::Element::from_str(s)?;
    println!("element: {e:?}");

    Ok(())
}

struct BtRoot {
    main_tree_to_execute: String,
    trees: HashMap<String, String>,
}

struct BtTree {
    id: String,
    node: Box<dyn TreeNode>,
}

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

// 只有action节点才是叶子节点
fn create_tree_node_recursively(factory: &Factory, s: &str) -> Result<Option<Box<dyn TreeNode>>> {
    println!("input: {s}");
    let mut reader = Reader::from_str(s);

    let mut control_nodes = VecDeque::new();

    loop {
        let event = reader.read_event();
        println!("event: {event:?}");

        match event {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let name = e.name();
                let element_name = std::str::from_utf8(name.as_ref())?;

                let wrapper = AttributesWrapper::new(e.attributes());

                if factory.composite_types().contains(element_name) {
                    println!("composite node");
                    let Some(node) = factory.build_composite(element_name, wrapper.kv()?) else {
                        continue;
                    };

                    control_nodes.push_front(node);
                } else if factory.decorator_types().contains(element_name) {
                    println!("decorator node");

                    match reader.read_event()? {
                        Event::Start(e) | Event::Empty(e) => {
                            let node_name = e.name();
                            let node_element_name = std::str::from_utf8(node_name.as_ref())?;
                            let node_wrapper = AttributesWrapper::new(e.attributes());

                            let Some(node) =
                                factory.build_action(node_element_name, node_wrapper.kv()?)
                            else {
                                continue;
                            };

                            let Some(node) =
                                factory.build_decorator(element_name, wrapper.kv()?, node)
                            else {
                                continue;
                            };

                            if let Some(control_node) = control_nodes.front_mut() {
                                control_node.add_child(node);
                            } else {
                                return Ok(Some(node));
                            }
                        }
                        _ => {}
                    }

                    let new_range = reader.read_to_end(e.to_end().name())?;

                    let node = create_tree_node_recursively(factory, &s[new_range.clone()])?;
                    println!("new range: {new_range:?} node= {}", node.is_none());
                } else {
                    println!("leaf node: {element_name}");
                    let Some(node) = factory.build_action(element_name, wrapper.kv()?) else {
                        continue;
                    };

                    if let Some(control_node) = control_nodes.front_mut() {
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
                    if let Some(control_node) = control_nodes.pop_front() {
                        if let Some(parent_control_node) = control_nodes.front_mut() {
                            parent_control_node.add_child(control_node);
                        } else {
                            return Ok(Some(control_node));
                        }
                    } else {
                        println!("unexpected end: {element_name}");
                    }
                }
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
    }

    Ok(None)
}

pub fn from_str(factory: &Factory, s: &str) -> Result<()> {
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

    let mut ctx = Context::default();

    for (id, tree_range) in tree_ranges {
        let node = create_tree_node_recursively(factory, &s[tree_range.clone()])?;

        if let Some(mut node) = node {
            let node_type = node.node_type();

            loop {
                let res = node.tick(&mut ctx);

                if res != NodeStatus::Running {
                    break;
                }
            }

            println!("debug: {}", node.debug_info());
        }
        println!("id= {id} content= {}", &s[tree_range]);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use crate::{factory::boxify_action, NodeStatus};

    use super::*;

    fn assets_dir() -> PathBuf {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("assets");

        d
    }

    struct PrintBody;

    impl TreeNode for PrintBody {
        fn tick(&mut self, ctx: &mut crate::Context) -> crate::NodeStatus {
            println!("body tick");
            NodeStatus::Success
        }
    }

    struct PrintArm;

    impl TreeNode for PrintArm {
        fn tick(&mut self, ctx: &mut crate::Context) -> NodeStatus {
            println!("arm tick");
            NodeStatus::Success
        }
    }

    const XML: &str = r#"
    <?xml version="1.0" encoding="UTF-8"?>
    <root BTCPP_format="4">
        <BehaviorTree ID="untitled">
            <Sequence>
                <PrintBody body="body"/>
                <PrintArm arm="left_arm"/>
                <PrintArm arm="{arm}"/>
                <Sequence>
                    <PrintBody body="body"/>
                    <PrintArm arm="arm"/>
                </Sequence>
            </Sequence>
        </BehaviorTree>
    </root>"#;

    #[test]
    fn test_parse() {
        // let mut xml_path = assets_dir();
        // xml_path.push("full.xml");
        // let str = std::fs::read_to_string(xml_path).unwrap();

        let mut factory = Factory::default();
        factory.register_action_node_type("PrintBody".to_string(), boxify_action(|_| PrintBody));
        factory.register_action_node_type("PrintArm".to_string(), boxify_action(|_| PrintArm));

        from_str(&factory, XML).unwrap();
    }
}
