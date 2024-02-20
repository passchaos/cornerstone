use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, Weak},
};

use parking_lot::{RwLock, RwLockWriteGuard};
use serde_json::Value;

pub mod action;
pub mod composite;
pub mod decorator;

#[derive(Default)]
pub struct Blackboard {
    storage: RwLock<HashMap<String, Value>>,
    parent_bb: Option<Weak<RwLock<Blackboard>>>,
    internal_to_external: RwLock<HashMap<String, String>>,
}

impl Blackboard {
    pub fn new_with_parent(parent_bb: &Arc<RwLock<Blackboard>>) -> Self {
        let parent_bb = Some(Arc::downgrade(parent_bb));

        Self {
            parent_bb,
            ..Default::default()
        }
    }

    pub fn get_entry(&self, key: &str) -> Option<Value> {
        self.storage.read().get(key).cloned()
    }

    pub fn set(&mut self, key: String, value: Value) {
        self.storage.write().insert(key, value);
    }
}

#[derive(Clone)]
pub struct DataProxy {
    bb: Arc<RwLock<Blackboard>>,
    input_ports: HashMap<String, String>,
    uid: u16,
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
    pub fn new(bb: Arc<RwLock<Blackboard>>) -> Self {
        Self {
            bb,
            input_ports: HashMap::new(),
            uid: 0,
        }
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
}
