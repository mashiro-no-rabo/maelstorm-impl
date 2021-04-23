use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Number as Jnum, Value as Jval};
use std::{
  collections::BTreeMap,
  io::{self, BufRead, BufReader, Stdin, Write},
  sync::atomic::{AtomicU64, Ordering},
  time::Duration,
};
use timeout_readwrite::TimeoutReader;

use maelstrom::*;

#[derive(Debug, Clone)]
enum Op {
  Append(u64, u64),
  // Verify(u64, Vec<u64>),
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
          // Jval::Array(expected) => Self::Verify(k, expected.iter().cloned().map(|v| v.as_u64().unwrap()).collect()),
          _ => unimplemented!("unexpected read function value"),
        }
      }
      _ => unimplemented!("unexpected transaction µ-op function"),
    }
  }
}

type Key = u64;
type Val = Vec<u64>;
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Database(BTreeMap<Key, Val>);

const DB_KEY: &str = "db";

impl Database {
  fn commit(&mut self, txn: &[Op]) -> Vec<Vec<Jval>> {
    txn
      .iter()
      .cloned()
      .map(|op| match op {
        Op::Append(k, v) => {
          if let Some(list) = self.0.get_mut(&k) {
            list.push(v);
          } else {
            self.0.insert(k, vec![v]);
          }

          vec![
            Jval::String("append".to_owned()),
            Jval::Number(Jnum::from(k)),
            Jval::Number(Jnum::from(v)),
          ]
        }
        Op::Read(k) => {
          vec![
            Jval::String("r".to_owned()),
            Jval::Number(Jnum::from(k)),
            self.0.get(&k).cloned().map_or(Jval::Null, |vals| {
              vals
                .into_iter()
                .map(|v| Jval::Number(Jnum::from(v)))
                .collect::<Vec<Jval>>()
                .into()
            }),
          ]
        }
      })
      .collect()
  }
}

fn main() -> Result<()> {
  // logging
  let stderr = io::stderr();
  let mut log = stderr.lock();

  // input
  let stdin = io::stdin();
  let mut stdin = BufReader::new(TimeoutReader::new(stdin, Duration::from_millis(500)));

  // msg_id generation
  let msg_id = AtomicU64::new(0);
  let gen_id = move || Some(msg_id.fetch_add(1, Ordering::SeqCst));

  // node data
  let mut node_id = String::new();

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
        "txn" => {
          let db1: Database = rpc(
            &mut stdin,
            Message {
              src: node_id.clone(),
              dest: "lin-kv".to_owned(),
              body: MsgBody {
                typ: "read".to_owned(),
                key: Some(DB_KEY.into()),
                ..Default::default()
              },
            },
          )
          .unwrap()
          .body
          .value
          .map_or(Default::default(), |x| {
            serde_json::from_str(x.as_str().unwrap()).unwrap()
          });

          let mut db2 = db1.clone();

          let t: Vec<Op> = msg.body.txn.clone().unwrap().iter().map(|x| Op::from_txn(x)).collect();
          let ret = db2.commit(&t);

          let cas_resp = rpc(
            &mut stdin,
            Message {
              src: node_id.clone(),
              dest: "lin-kv".to_owned(),
              body: MsgBody {
                typ: "cas".to_owned(),
                key: Some(DB_KEY.into()),
                from: Some(serde_json::to_string(&db1).unwrap().into()),
                to: Some(serde_json::to_string(&db2).unwrap().into()),
                create_if_not_exists: Some(true),
                ..Default::default()
              },
            },
          );

          if cas_resp.map_or(false, |cr| cr.body.typ == "cas_ok") {
            let r = MsgBody {
              typ: "txn_ok".to_owned(),
              msg_id: gen_id(),
              txn: Some(ret),
              ..Default::default()
            };
            reply(&msg, r)?;
          } else {
            let r = MsgBody {
              typ: "error".to_owned(),
              msg_id: gen_id(),
              code: Some(30),
              text: Some("CAS failed".to_owned()),
              ..Default::default()
            };
            reply(&msg, r)?;
          }
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

fn rpc(stdin: &mut BufReader<TimeoutReader<Stdin>>, req: Message) -> Result<Message> {
  println!("{}", serde_json::to_string(&req)?);

  let mut input = String::new();
  let _ = stdin.read_line(&mut input)?;

  let msg: Message = serde_json::from_str(&input)?;
  Ok(msg)
}
