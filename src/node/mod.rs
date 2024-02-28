use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, Weak},
};

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use serde_json::Value;

use crate::NodeStatus;

pub mod action;
pub mod composite;
pub mod decorator;

#[derive(Default)]
pub struct Blackboard {
    storage: RwLock<HashMap<String, Value>>,
    parent_bb: Option<Weak<RwLock<Blackboard>>>,
    internal_to_external: RwLock<HashMap<String, String>>,
}

impl std::fmt::Debug for Blackboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Blackboard")
            .field(
                "parent_bb",
                &self.parent_bb.as_ref().and_then(|pb| pb.upgrade()),
            )
            .field("internal_to_external", &self.internal_to_external)
            .finish()
    }
}

impl Blackboard {
    pub fn extend_parent_remappings(&mut self, remappings: HashMap<String, String>) {
        self.internal_to_external.write().extend(remappings);
    }

    pub fn port_remappings(&self) -> RwLockReadGuard<HashMap<String, String>> {
        self.internal_to_external.read()
    }

    pub fn new_with_parent(parent_bb: &Arc<RwLock<Blackboard>>) -> Self {
        let parent_bb = Some(Arc::downgrade(parent_bb));

        Self {
            parent_bb,
            ..Default::default()
        }
    }

    pub fn get_entry(&self, key: &str) -> Option<Value> {
        if let Some(v) = self.storage.read().get(key).cloned() {
            Some(v)
        } else {
            let i_to_e_guard = self.internal_to_external.read();

            let parent_key = if let Some(external_key) = i_to_e_guard.get(key) {
                external_key
            } else {
                key
            };

            if let Some(parent_bb) = self.parent_bb.as_ref().and_then(|a| a.upgrade()) {
                let value = parent_bb.read().get_entry(parent_key);

                value
            } else {
                None
            }
        }
    }

    pub fn set(&mut self, key: String, value: Value) {
        tracing::trace!("set blackboard: key= {key} value= {value:?}");

        self.storage.write().insert(key, value);
    }
}

#[derive(Clone)]
pub struct DataProxy {
    bb: Arc<RwLock<Blackboard>>,
    input_ports: HashMap<String, String>,
    status: NodeStatus,
    uid: u16,
    path: String,
}

impl std::fmt::Debug for DataProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataProxy")
            .field("keys", &self.input_ports.keys())
            .finish()
    }
}

pub fn is_ref_key(key: &str) -> bool {
    key.starts_with("{") && key.ends_with("}")
}

pub fn strip_ref_tag(key: &str) -> String {
    key.replace("{", "").replace("}", "")
}

impl DataProxy {
    pub fn set_path(&mut self, path: String) {
        self.path = path;
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn new(bb: Arc<RwLock<Blackboard>>) -> Self {
        Self::new_with_uid(0, bb, HashMap::new())
    }

    pub fn new_with_uid(
        uid: u16,
        bb: Arc<RwLock<Blackboard>>,
        input_ports: HashMap<String, String>,
    ) -> Self {
        Self {
            bb,
            input_ports,
            status: NodeStatus::default(),
            uid,
            path: String::new(),
        }
    }

    pub fn add_input(&mut self, key: String, value: String) {
        self.input_ports.insert(key, value);
    }

    pub fn get_input<T: FromStr>(&self, key: &str) -> Option<T>
    where
        for<'de> T: serde::Deserialize<'de>,
    {
        let Some(input_value_str) = self.input_ports.get(key) else {
            return None;
        };

        if is_ref_key(&input_value_str) {
            let stripped_key = strip_ref_tag(&input_value_str);

            let Some(bb_value) = self.bb.read().get_entry(&stripped_key) else {
                return None;
            };

            serde_json::from_value(bb_value).ok()
        } else {
            input_value_str.parse().ok()
        }
    }

    pub fn set_uid(&mut self, uid: u16) {
        self.uid = uid;
    }

    pub fn uid(&self) -> u16 {
        self.uid
    }

    pub fn blackboard(&self) -> RwLockWriteGuard<Blackboard> {
        self.bb.write()
    }

    pub fn reset_status(&mut self) {
        self.status = NodeStatus::Idle;
    }

    pub fn status(&self) -> NodeStatus {
        self.status
    }
}
