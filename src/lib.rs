use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
  pub src: String,
  pub dest: String,
  pub body: MsgBody,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MsgBody {
  #[serde(rename = "type")]
  pub typ: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub msg_id: Option<u64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub in_reply_to: Option<u64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub echo: Option<String>,
}
