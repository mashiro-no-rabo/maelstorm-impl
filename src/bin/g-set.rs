use anyhow::Result;
use std::{
  collections::HashSet,
  io::{self, Write},
  sync::{
    atomic::{AtomicU64, Ordering},
    Arc, RwLock,
  },
  thread,
  time::Duration,
};

use maelstrom::*;

struct GSet(HashSet<u64>);

impl CRDT for GSet {
  type Element = u64;
  type Value = HashSet<u64>;

  fn init() -> Self {
    GSet(HashSet::new())
  }

  fn add(&mut self, val: Self::Element) {
    self.0.insert(val);
  }

  fn read(&self) -> Self::Value {
    self.0.clone()
  }

  fn merge(&mut self, other: &Self) {
    *self = GSet(self.0.union(&other.0).cloned().collect())
  }

  fn from_msg_body(mb: &MsgBody) -> Self {
    GSet(mb.value.clone().unwrap())
  }

  fn into_msg_body(&self) -> MsgBody {
    MsgBody {
      value: Some(self.0.clone()),
      ..Default::default()
    }
  }
}

fn main() -> Result<()> {
  // logging
  let stderr = io::stderr();
  let mut log = stderr.lock();

  // input
  let stdin = io::stdin();

  // msg_id generation
  let msg_id = AtomicU64::new(0);
  let gen_id = move || Some(msg_id.fetch_add(1, Ordering::SeqCst));

  // node data
  let gset = Arc::new(RwLock::new(GSet::init()));

  loop {
    let mut input = String::new();
    if let Ok(_) = stdin.read_line(&mut input) {
      // log.write_all(format!("Raw message: {}\n", input.trim()).as_bytes())?;
      let msg: Message = serde_json::from_str(&input)?;
      // log.write_all(format!("Parse result: {:?}\n", &msg).as_bytes())?;

      match msg.body.typ.as_str() {
        "init" => {
          let node_id = msg.body.node_id.clone().unwrap();
          let other_nodes: Vec<String> = msg
            .body
            .node_ids
            .clone()
            .unwrap()
            .into_iter()
            .filter(|n| n != &node_id)
            .collect();

          log.write_all(format!("Node {} initialized\n", &node_id).as_bytes())?;

          // replicate thread
          let set_reader = gset.clone();
          thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(2_000));

            let mut bd = set_reader.read().unwrap().into_msg_body();
            bd.typ = "replicate".to_owned();

            for dest in other_nodes.iter().cloned() {
              let msg = Message {
                src: node_id.clone(),
                dest,
                body: bd.clone(),
              };

              println!("{}", serde_json::to_string(&msg).unwrap());
            }
          });

          let r = MsgBody {
            typ: "init_ok".to_owned(),
            msg_id: gen_id(),
            ..Default::default()
          };
          reply(&msg, r)?;
        }
        "add" => {
          let elem = msg.body.element.unwrap();
          {
            let mut set_writer = gset.write().unwrap();
            set_writer.add(elem);
          }

          let r = MsgBody {
            typ: "add_ok".to_owned(),
            msg_id: gen_id(),
            ..Default::default()
          };
          reply(&msg, r)?;
        }
        "replicate" => {
          let other = GSet::from_msg_body(&msg.body);

          let mut set_writer = gset.write().unwrap();
          set_writer.merge(&other);
        }
        "read" => {
          let r = MsgBody {
            typ: "read_ok".to_owned(),
            msg_id: gen_id(),
            value: Some(gset.read().unwrap().read()),
            ..Default::default()
          };

          reply(&msg, r)?;
        }
        _ => unimplemented!("unexpected message"),
      }
    }
  }
}

fn reply(origin: &Message, mut resp_body: MsgBody) -> Result<()> {
  resp_body.in_reply_to = Some(origin.body.msg_id.unwrap());

  let reply = Message {
    src: origin.dest.clone(),
    dest: origin.src.clone(),
    body: resp_body,
  };

  println!("{}", serde_json::to_string(&reply)?);

  Ok(())
}
