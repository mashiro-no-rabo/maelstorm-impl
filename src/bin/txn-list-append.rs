use anyhow::{bail, Result};
use serde_json::{Number as Jnum, Value as Jval};
use std::{
  collections::HashMap,
  io::{self, Write},
  sync::atomic::{AtomicU64, Ordering},
};

use maelstrom::*;

#[derive(Debug, Clone)]
enum Op {
  Append(u64, u64),
  Verify(u64, Vec<u64>),
  Read(u64),
}

impl Op {
  fn from_txn(txn: &[Jval]) -> Self {
    let mut iter = txn.iter();
    match iter.next().unwrap().as_str().unwrap() {
      "append" => Self::Append(
        iter.next().unwrap().as_u64().unwrap(),
        iter.next().unwrap().as_u64().unwrap(),
      ),
      "r" => {
        let k = iter.next().unwrap().as_u64().unwrap();
        match iter.next().unwrap() {
          Jval::Null => Self::Read(k),
          Jval::Array(expected) => Self::Verify(k, expected.iter().cloned().map(|v| v.as_u64().unwrap()).collect()),
          _ => unimplemented!("unexpected read function value"),
        }
      }
      _ => unimplemented!("unexpected transaction µ-op function"),
    }
  }
}

type Key = u64;
type Val = Vec<u64>;
#[derive(Debug, Clone, Default)]
struct Database(HashMap<Key, Val>);

impl Database {
  fn commit(&mut self, txn: &[Op]) -> Result<Vec<Vec<Jval>>> {
    let mut ret = Vec::new();
    for op in txn.iter().cloned() {
      match op {
        Op::Append(k, v) => {
          if let Some(list) = self.0.get_mut(&k) {
            list.push(v);
          } else {
            self.0.insert(k, vec![v]);
          }

          ret.push(vec![
            Jval::String("append".to_owned()),
            Jval::Number(Jnum::from(k)),
            Jval::Number(Jnum::from(v)),
          ]);
        }
        Op::Verify(k, expected) => {
          if let Some(list) = self.0.get(&k) {
            if list == &expected {
              ret.push(vec![
                Jval::String("r".to_owned()),
                Jval::Number(Jnum::from(k)),
                Jval::Array(
                  expected
                    .clone()
                    .into_iter()
                    .map(|v| Jval::Number(Jnum::from(v)))
                    .collect(),
                ),
              ]);
            } else {
              bail!("failed to verify");
            }
          } else {
            bail!("failed to verify");
          }
        }
        Op::Read(k) => {
          ret.push(vec![
            Jval::String("r".to_owned()),
            Jval::Number(Jnum::from(k)),
            self.0.get(&k).cloned().map_or(Jval::Null, |vals| {
              vals
                .into_iter()
                .map(|v| Jval::Number(Jnum::from(v)))
                .collect::<Vec<Jval>>()
                .into()
            }),
          ]);
        }
      }
    }

    Ok(ret)
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
  let mut db = Database::default();

  loop {
    let mut input = String::new();
    if let Ok(_) = stdin.read_line(&mut input) {
      // log.write_all(format!("Raw message: {}\n", input.trim()).as_bytes())?;
      let msg: Message = serde_json::from_str(&input)?;
      // log.write_all(format!("Parse result: {:?}\n", &msg).as_bytes())?;

      match msg.body.typ.as_str() {
        "init" => {
          let node_id = msg.body.node_id.clone().unwrap();
          let _other_nodes: Vec<String> = msg
            .body
            .node_ids
            .clone()
            .unwrap()
            .into_iter()
            .filter(|n| n != &node_id)
            .collect();

          log.write_all(format!("Node {} initialized\n", &node_id).as_bytes())?;

          let r = MsgBody {
            typ: "init_ok".to_owned(),
            msg_id: gen_id(),
            ..Default::default()
          };
          reply(&msg, r)?;
        }
        "txn" => {
          let t: Vec<Op> = msg.body.txn.clone().unwrap().iter().map(|x| Op::from_txn(x)).collect();
          let ret = db.commit(&t)?;

          let r = MsgBody {
            typ: "txn_ok".to_owned(),
            msg_id: gen_id(),
            txn: Some(ret),
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
