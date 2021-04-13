use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
  pub src: String,
  pub dest: String,
  pub body: MsgBody,
}

type NodeID = String;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MsgBody {
  #[serde(rename = "type")]
  pub typ: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub msg_id: Option<u64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub in_reply_to: Option<u64>,
  // init
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_id: Option<NodeID>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_ids: Option<Vec<NodeID>>,
  // echo workload
  #[serde(skip_serializing_if = "Option::is_none")]
  pub echo: Option<Value>,
  // broadcast
  #[serde(skip_serializing_if = "Option::is_none")]
  pub topology: Option<HashMap<NodeID, Vec<NodeID>>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub message: Option<Value>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub messages: Option<Vec<Value>>,
  // g-set
  #[serde(skip_serializing_if = "Option::is_none")]
  pub element: Option<Value>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub value: Option<Value>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub set: Option<HashSet<u64>>,
  // g-counter
  #[serde(skip_serializing_if = "Option::is_none")]
  pub delta: Option<Value>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub counters: Option<HashMap<String, u64>>,
  // pn-counter
  #[serde(skip_serializing_if = "Option::is_none")]
  pub pn_counters: Option<(HashMap<String, u64>, HashMap<String, u64>)>,
}

mod crdt;
pub use crdt::*;
