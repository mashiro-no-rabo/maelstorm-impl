use anyhow::Result;
use std::{
  collections::HashMap,
  io,
  sync::{
    atomic::{AtomicU64, Ordering},
    mpsc::{self, Receiver, Sender},
  },
  thread,
};

use maelstrom::*;

fn main() -> Result<()> {
  // input
  let stdin = io::stdin();

  // nodes -> tx channels
  let mut node_chans: HashMap<String, Sender<Message>> = HashMap::new();

  // master loop
  loop {
    let mut input = String::new();
    if let Ok(_) = stdin.read_line(&mut input) {
      let msg: Message = serde_json::from_str(&input)?;
      log(format!("Master loop got: {}", input))?;

      if msg.body.typ == "init" {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || node(rx).unwrap());
        node_chans.insert(msg.body.node_id.clone().unwrap(), tx);
      }

      if let Some(tx) = node_chans.get(&msg.dest) {
        tx.send(msg.clone())?;
      } else {
        unimplemented!("node not found!")
      }
    }
  }
}

fn node(rx: Receiver<Message>) -> Result<()> {
  let msg_id = AtomicU64::new(0);

  let mut node_id = String::new();
  let mut neighbors: Vec<String> = vec![];

  while let Ok(msg) = rx.recv() {
    match msg.body.typ.as_str() {
      "init" => {
        node_id = msg.body.node_id.clone().unwrap();
        log(format!("Node {} initialized", &node_id))?;

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

  Ok(())
}

fn log(l: String) -> Result<()> {
  use std::io::Write;

  let stderr = io::stderr();
  let mut log = stderr.lock();
  log.write_all(l.as_bytes())?;
  log.flush()?;

  Ok(())
}
