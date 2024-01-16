use std::{collections::HashMap, fs::read, hash::Hash, ops::Range, str::FromStr};

use crate::{
    factory::{composite_node_types, decorator_node_types, Factory},
    node::composite::ControlNode,
    Result, TreeNode,
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
}

fn create_tree_node_recursively<P: ControlNode>(
    node_factory: &Factory,
    parent_node: Option<&mut P>,
    s: &str,
    range: Range<usize>,
) -> Result<Option<Box<dyn TreeNode>>> {
    let mut reader = Reader::from_str(&s[range]);

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = e.name();
                let element_name = std::str::from_utf8(name.as_ref())?;

                if composite_node_types().contains(element_name) {
                    let Some(mut node) = node_factory.build_action(element_name) else {
                        continue;
                    };

                    let range = reader.read_to_end(e.to_end().to_owned().name())?;

                //     let child_node = create_tree_node_recursively(node_factory, s, range)?;

                //     if let Some(node) = node {

                //     }
                } else if decorator_node_types().contains(element_name) {
                    let Some(mut node) = node_factory.build_action(element_name) else {
                        continue;
                    };
                } else {
                    let Some(node) = node_factory.build_action(element_name) else {
                        continue;
                    };

                    return Ok(Some(node));
                }

                let decorator_types = decorator_node_types();

                // if e.name().as_ref()
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
    }

    Ok(None)
}

pub fn from_str(s: &str) -> Result<()> {
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

    for (id, tree_range) in tree_ranges {
        println!("id= {id} content= {}", &s[tree_range]);
    }

    // let res = reader.read_to_end(QName(b"root"))?;

    // let mut root = loop {
    //     }

    //     return Ok(())
    // }

    // loop {
    //     match reader.read_event_into(&mut buf) {
    //         Ok(Event::Start(e)) if e.name().as_ref() == b"root" => {
    //             println!("get root: {e:?}");

    //             let mut inner_buf = Vec::new();
    //             let end = e.to_end().to_owned();
    //             let res = reader.read_to_end_into(end.name(), &mut inner_buf)?;

    //             let tree = &s[res.clone()];
    //             println!("tree: {tree}");
    //             println!("root res= {res:?} inner= {inner_buf:?}");

    //         }
    //         Ok(Event::Start(e)) => {
    //             println!("start: {e:?}");

    //             let mut inner_buf = Vec::new();
    //             let end = e.to_end().to_owned();
    //             let res = reader.read_to_end_into(end.name(), &mut inner_buf);

    //             println!("res= {res:?} inner= {inner_buf:?}");
    //             handle_byte_start(e);
    //         }
    //         Ok(Event::End(e)) => {
    //             println!("end: {e:?}");
    //         }
    //         Ok(Event::Empty(e)) => {
    //             println!("empty: {e:?}");
    //             handle_byte_start(e);
    //         }
    //         Ok(Event::Text(e)) => {
    //             println!("text: {e:?}");
    //         }
    //         Ok(Event::CData(e)) => {
    //             println!("cdata: {e:?}");
    //         }
    //         Ok(Event::Comment(e)) => {
    //             println!("comment: {e:?}");
    //         }
    //         Ok(Event::Decl(e)) => {
    //             println!("decl: {e:?}");
    //         }
    //         Ok(Event::PI(e)) => {
    //             println!("pi: {e:?}");
    //         }
    //         Ok(Event::DocType(e)) => {
    //             println!("doctype: {e:?}");
    //         }
    //         Ok(Event::Eof) => break,
    //         Err(e) => {
    //             println!("meet error: {e:?}");
    //         }
    //         _ => (),
    //     }
    // }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;

    fn assets_dir() -> PathBuf {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("assets");

        d
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
                <AlwaysSuccess/>
            </Sequence>
        </BehaviorTree>
    </root>"#;

    #[test]
    fn test_parse() {
        let mut xml_path = assets_dir();
        xml_path.push("full.xml");
        let str = std::fs::read_to_string(xml_path).unwrap();

        from_str(&str).unwrap();
    }
}
