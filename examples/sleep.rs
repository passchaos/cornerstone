use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use cornerstone::{
    node::control::{ControlNode, SequenceNode},
    Context, DataProxy, NodeStatus, TreeNode,
};

struct SleepNode {
    end_ts: Instant,
    data_proxy: DataProxy,
}

impl TreeNode for SleepNode {
    fn tick(&mut self, ctx: &mut cornerstone::Context) -> NodeStatus {
        let current_ts = Instant::now();

        if current_ts <= self.end_ts {
            NodeStatus::Running
        } else {
            NodeStatus::Success
        }
    }
}

fn main() {
    let mut ctx = Context::default();

    let sleep_node = SleepNode {
        end_ts: Instant::now() + Duration::from_secs(3),
        data_proxy: DataProxy::new(HashMap::new()),
    };

    let mut root = SequenceNode::default();
    root.add_child(Box::new(sleep_node));

    loop {
        let res = root.tick(&mut ctx);

        if res != NodeStatus::Running {
            println!("finish run sleep node: res= {res:?}");
            break;
        } else {
            println!("need wait for finish");
            std::thread::sleep(Duration::from_millis(200));
        }
    }
}
