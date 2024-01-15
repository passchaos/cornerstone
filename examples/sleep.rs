use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use cornerstone::{
    node::control::{ControlNode, Parallel, Sequence},
    Context, DataProxy, NodeStatus, TreeNode,
};

struct SleepNode {
    name: String,
    end_ts: Instant,
    data_proxy: DataProxy,
}

impl TreeNode for SleepNode {
    fn tick(&mut self, ctx: &mut cornerstone::Context) -> NodeStatus {
        let current_ts = Instant::now();

        if current_ts <= self.end_ts {
            println!("sleep: {}", self.name);

            NodeStatus::Running
        } else {
            println!("finish: {}", self.name);
            NodeStatus::Success
        }
    }
}

fn main() {
    let mut ctx = Context::default();

    let sleep_node_1 = SleepNode {
        name: "alice".to_string(),
        end_ts: Instant::now() + Duration::from_secs(3),
        data_proxy: DataProxy::new(HashMap::new()),
    };

    let sleep_node_2 = SleepNode {
        name: "bob".to_string(),
        end_ts: Instant::now() + Duration::from_secs(5),
        data_proxy: DataProxy::new(HashMap::new()),
    };

    let mut root = Parallel::new(Some(1), None);
    root.add_child(Box::new(sleep_node_1));
    root.add_child(Box::new(sleep_node_2));

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
