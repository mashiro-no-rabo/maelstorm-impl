use anyhow::Result;
use std::{
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

  // node data
  let msg_id = AtomicU64::new(0);
  let mut node_id = String::new();
  let mut neighbors: Vec<String> = vec![];

  // master loop
  loop {
    let mut input = String::new();
    if let Ok(_) = stdin.read_line(&mut input) {
      let msg: Message = serde_json::from_str(&input)?;
      log.write_all(format!("Got: {:?}\n", &msg).as_bytes())?;

      match msg.body.typ.as_str() {
        "init" => {
          node_id = msg.body.node_id.clone().unwrap();
          log.write_all(format!("Node {} initialized\n", &node_id).as_bytes())?;

          let reply = Message {
            src: msg.dest,
            dest: msg.src,
            body: MsgBody {
              typ: "init_ok".to_owned(),
              msg_id: Some(msg_id.fetch_add(1, Ordering::SeqCst)),
              in_reply_to: Some(msg.body.msg_id.unwrap()),
              ..Default::default()
            },
          };

          println!("{}", serde_json::to_string(&reply)?);
        }
        "topology" => {
          // parse neighbors
          neighbors = msg.body.topology.unwrap().get(&node_id).unwrap().clone();
          log.write_all(format!("Neighbors: {:?}\n", &neighbors).as_bytes())?;

          let reply = Message {
            src: msg.dest,
            dest: msg.src,
            body: MsgBody {
              typ: "topology_ok".to_owned(),
              msg_id: Some(msg_id.fetch_add(1, Ordering::SeqCst)),
              in_reply_to: Some(msg.body.msg_id.unwrap()),
              ..Default::default()
            },
          };

          println!("{}", serde_json::to_string(&reply)?);
        }
        _ => unimplemented!("unexpected message"),
      }
    }
  }
}
