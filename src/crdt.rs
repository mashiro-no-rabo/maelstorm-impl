use super::MsgBody;

pub trait CRDT {
  type Element;

  fn init() -> Self;

  fn from_msg_body(_: &MsgBody) -> Self;
  fn into_msg_body(&self) -> MsgBody;

  fn add(&mut self, val: Self::Element);
  fn merge(&mut self, other: &Self);
}
