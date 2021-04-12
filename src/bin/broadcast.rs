use anyhow::Result;
use std::{
  collections::HashSet,
  io::{self, Write},
  sync::atomic::{AtomicU64, Ordering},
};

use maelstrom::*;

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
  let mut node_id = String::new();
  let mut neighbors: Vec<String> = vec![];
  let mut messages: HashSet<u64> = HashSet::new();

  loop {
    let mut input = String::new();
    if let Ok(_) = stdin.read_line(&mut input) {
      // log.write_all(format!("Raw message: {}\n", input.trim()).as_bytes())?;
      let msg: Message = serde_json::from_str(&input)?;
      // log.write_all(format!("Parse result: {:?}\n", &msg).as_bytes())?;

      match msg.body.typ.as_str() {
        "init" => {
          node_id = msg.body.node_id.clone().unwrap();
          log.write_all(format!("Node {} initialized\n", &node_id).as_bytes())?;

          let r = MsgBody {
            typ: "init_ok".to_owned(),
            msg_id: gen_id(),
            ..Default::default()
          };
          reply(&msg, r)?;
        }
        "topology" => {
          // parse neighbors
          neighbors = msg.body.topology.clone().unwrap().get(&node_id).unwrap().clone();
          log.write_all(format!("Neighbors: {:?}\n", &neighbors).as_bytes())?;

          let r = MsgBody {
            typ: "topology_ok".to_owned(),
            msg_id: gen_id(),
            ..Default::default()
          };
          reply(&msg, r)?;
        }
        "broadcast" => {
          let msg_content = msg.body.message.unwrap();
          if messages.insert(msg_content) {
            // new message, gossip
            for nb in neighbors.iter() {
              let gossip = Message {
                src: node_id.clone(),
                dest: nb.clone(),
                body: MsgBody {
                  typ: "broadcast".to_owned(),
                  msg_id: gen_id(),
                  message: Some(msg_content),
                  ..Default::default()
                },
              };
              println!("{}", serde_json::to_string(&gossip)?);
            }
          }

          let r = MsgBody {
            typ: "broadcast_ok".to_owned(),
            msg_id: gen_id(),
            ..Default::default()
          };
          reply(&msg, r)?;
        }
        "broadcast_ok" => {
          // ignore gossip responses
        }
        "read" => {
          let r = MsgBody {
            typ: "read_ok".to_owned(),
            msg_id: gen_id(),
            messages: Some(messages.iter().cloned().collect()),
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
