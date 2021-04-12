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
  let set = Arc::new(RwLock::new(HashSet::<u64>::new()));

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
          let set_reader = set.clone();
          thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(2_000));

            let bd = MsgBody {
              typ: "replicate".to_owned(),
              value: Some(set_reader.read().unwrap().iter().cloned().collect()),
              ..Default::default()
            };

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
            let mut set_writer = set.write().unwrap();
            set_writer.insert(elem);
          }

          let r = MsgBody {
            typ: "add_ok".to_owned(),
            msg_id: gen_id(),
            ..Default::default()
          };
          reply(&msg, r)?;
        }
        "replicate" => {
          let mut set_writer = set.write().unwrap();
          *set_writer = set_writer.union(&msg.body.value.unwrap()).cloned().collect();
        }
        "read" => {
          let r = MsgBody {
            typ: "read_ok".to_owned(),
            msg_id: gen_id(),
            value: Some(set.read().unwrap().iter().cloned().collect()),
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
