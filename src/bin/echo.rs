use anyhow::Result;
use std::io::{self, Write};

use maelstrom::*;

fn main() -> Result<()> {
  let stderr = io::stderr();
  let mut out = stderr.lock();
  let stdin = io::stdin();

  let mut next_msg_id: u64 = 0;
  loop {
    let mut input = String::new();
    if let Ok(_) = stdin.read_line(&mut input) {
      out.write_all(format!("Received: {}", input).as_bytes())?;

      let msg: Message = serde_json::from_str(&input)?;
      match msg.body.typ.as_str() {
        "init" => {
          let reply = Message {
            src: msg.dest,
            dest: msg.src,
            body: MsgBody {
              typ: "init_ok".to_owned(),
              msg_id: Some(gen_msg_id(&mut next_msg_id)),
              in_reply_to: Some(msg.body.msg_id.unwrap()),
              ..Default::default()
            },
          };

          println!("{}", serde_json::to_string(&reply)?);
        }
        "echo" => {
          let reply = Message {
            src: msg.dest,
            dest: msg.src,
            body: MsgBody {
              typ: "echo_ok".to_owned(),
              msg_id: Some(gen_msg_id(&mut next_msg_id)),
              in_reply_to: Some(msg.body.msg_id.unwrap()),
              echo: Some(msg.body.echo.unwrap()),
              ..Default::default()
            },
          };

          println!("{}", serde_json::to_string(&reply)?);
        }
        _ => {}
      }
    }
  }
}

fn gen_msg_id(next_id: &mut u64) -> u64 {
  let cp = *next_id;
  *next_id += 1;
  cp
}
