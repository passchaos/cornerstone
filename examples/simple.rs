use std::{collections::HashMap, sync::Arc};

use cornerstone::{
    node::{composite::Sequence, decorator::Repeat, Blackboard, DataProxy},
    NodeStatus, TreeNode,
};

#[derive(Clone, Debug)]
struct Arm {
    name: String,
}

#[derive(Debug)]
struct PrintArmNode {
    data_proxy: DataProxy,
}

impl TreeNode for PrintArmNode {
    fn tick(&mut self) -> NodeStatus {
        NodeStatus::Success
    }
}

#[derive(Debug)]
struct Body {
    left_arm: Arm,
    right_arm: Arm,
}

#[derive(Debug)]
struct PrintBodyNode {
    data_proxy: DataProxy,
}

impl TreeNode for PrintBodyNode {
    fn tick(&mut self) -> NodeStatus {
        NodeStatus::Failure
    }
}

fn main() {
    // let mut seq_root = Sequence::default();

    // let bb = Arc::new(Blackboard::default());

    // let body = PrintBodyNode {
    //     data_proxy: DataProxy::new(bb.clone()),
    // };
    // seq_root.add_child(Box::new(body));

    // let mut arms_root = Sequence::default();

    // let mut left_ports = HashMap::new();
    // left_ports.insert("arm".to_string(), "{left_arm}".to_string());
    // let left_arm = PrintArmNode {
    //     data_proxy: DataProxy::new(left_ports),
    // };

    // let mut right_ports = HashMap::new();
    // right_ports.insert("arm".to_string(), "{right_arm}".to_string());
    // let right_arm = PrintArmNode {
    //     data_proxy: DataProxy::new(right_ports),
    // };

    // arms_root.add_child(Box::new(left_arm));
    // arms_root.add_child(Box::new(right_arm));

    // seq_root.add_child(Box::new(arms_root));

    // let mut n_dp = DataProxy::default();
    // n_dp.insert(
    //     "num_cycles".to_string(),
    //     ProxyValue::Real(Box::new("10".to_string())),
    // );

    // let mut root = Repeat::new_with_count(5, n_dp, Box::new(seq_root));

    // loop {
    //     let status = root.tick();

    //     if status == NodeStatus::Success {
    //         println!("tree finish");
    //         break;
    //     }
    // }
}
