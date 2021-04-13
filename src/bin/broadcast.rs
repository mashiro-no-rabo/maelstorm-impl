use anyhow::Result;
use std::{
  collections::{HashMap, HashSet},
  io::{self, Write},
  sync::{
    atomic::{AtomicU64, Ordering},
    mpsc::{self, Sender},
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
  let mut node_id = String::new();
  let mut neighbors: Vec<String> = vec![];
  let mut messages: HashSet<u64> = HashSet::new();

  // track retry threads per msg_id (u64)
  let mut gossiping: HashMap<u64, Sender<()>> = HashMap::new();

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
          let msg_content = msg.body.message.clone().unwrap().as_u64().unwrap();
          if messages.insert(msg_content) {
            // new message, gossip
            for nb in neighbors.iter().filter(|nb| *nb != &msg.src) {
              let (tx, rx) = mpsc::channel();
              let msg_id = gen_id();
              let src = node_id.clone();
              let dest = nb.clone();

              thread::spawn(move || {
                loop {
                  let gossip = Message {
                    src: src.clone(),
                    dest: dest.clone(),
                    body: MsgBody {
                      typ: "broadcast".to_owned(),
                      msg_id,
                      message: Some(serde_json::Value::Number(serde_json::Number::from(msg_content))),
                      ..Default::default()
                    },
                  };
                  println!("{}", serde_json::to_string(&gossip).unwrap());

                  // wait for response
                  if let Ok(_) = rx.recv_timeout(Duration::from_millis(500)) {
                    break;
                  }
                }
              });

              gossiping.insert(msg_id.unwrap(), tx);
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
          // cancel gossip thread
          if let Some(tx) = gossiping.remove(&msg.body.in_reply_to.unwrap()) {
            tx.send(())?;
          }
        }
        "read" => {
          let r = MsgBody {
            typ: "read_ok".to_owned(),
            msg_id: gen_id(),
            messages: Some(
              messages
                .iter()
                .map(|x| serde_json::Value::Number(serde_json::Number::from(*x)))
                .collect(),
            ),
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
