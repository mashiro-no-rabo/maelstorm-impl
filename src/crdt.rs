use super::MsgBody;

pub trait CRDT {
  type Element;
  type Value;

  fn init() -> Self;

  fn add(&mut self, val: Self::Element);

  // final value
  fn read(&self) -> Self::Value;

  // merge
  fn merge(&mut self, other: &Self);
  fn from_msg_body(_: &MsgBody) -> Self;
  fn into_msg_body(&self) -> MsgBody;
}
