use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
  pub src: String,
  pub dest: String,
  pub body: MsgBody,
}

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
  pub node_id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_ids: Option<Vec<String>>,
  // echo workload
  #[serde(skip_serializing_if = "Option::is_none")]
  pub echo: Option<String>,
  // broadcast
  #[serde(skip_serializing_if = "Option::is_none")]
  pub topology: Option<HashMap<String, Vec<String>>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub message: Option<u64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub messages: Option<Vec<u64>>,
  // g-set
  #[serde(skip_serializing_if = "Option::is_none")]
  pub element: Option<u64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub value: Option<HashSet<u64>>,
}
