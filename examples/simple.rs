use std::collections::HashMap;

use cornerstone::{
    node::{
        composite::{Composite, Sequence},
        decorator::Repeat,
    },
    Context, DataProxy, NodeStatus, ProxyValue, TreeNode,
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
    fn tick(&mut self, ctx: &mut Context) -> NodeStatus {
        if let Some(arm) = self.data_proxy.get::<Arm>(ctx, "arm") {
            println!("get arm: {arm:?}");
        };

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
    fn tick(&mut self, ctx: &mut Context) -> NodeStatus {
        if let Some(body) = self.data_proxy.get::<Body>(ctx, "body") {
            println!("PrintBodyNode: {body:?}");

            let left_arm = body.left_arm.clone();
            let right_arm = body.right_arm.clone();

            ctx.set("left_arm".to_string(), left_arm);
            ctx.set("right_arm".to_string(), right_arm);

            NodeStatus::Success
        } else {
            NodeStatus::Failure
        }
    }
}

fn main() {
    let mut ctx = Context::default();
    ctx.set(
        "body".to_string(),
        Body {
            left_arm: Arm {
                name: "left_arm".to_string(),
            },
            right_arm: Arm {
                name: "right_arm".to_string(),
            },
        },
    );

    let mut seq_root = Sequence::default();

    let body = PrintBodyNode {
        data_proxy: DataProxy::new(HashMap::new()),
    };
    seq_root.add_child(Box::new(body));

    let mut arms_root = Sequence::default();

    let mut left_ports = HashMap::new();
    left_ports.insert("arm".to_string(), "{left_arm}".to_string());
    let left_arm = PrintArmNode {
        data_proxy: DataProxy::new(left_ports),
    };

    let mut right_ports = HashMap::new();
    right_ports.insert("arm".to_string(), "{right_arm}".to_string());
    let right_arm = PrintArmNode {
        data_proxy: DataProxy::new(right_ports),
    };

    arms_root.add_child(Box::new(left_arm));
    arms_root.add_child(Box::new(right_arm));

    seq_root.add_child(Box::new(arms_root));

    let mut n_dp = DataProxy::default();
    n_dp.insert(
        "num_cycles".to_string(),
        ProxyValue::Real(Box::new("10".to_string())),
    );

    let mut root = Repeat::new_with_count(5, n_dp, Box::new(seq_root));

    loop {
        let status = root.tick(&mut ctx);

        if status == NodeStatus::Success {
            println!("tree finish");
            break;
        }
    }
}
