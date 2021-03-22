use anyhow::Result;
use std::io::{self, Write};

fn main() -> Result<()> {
  let stderr = io::stderr();
  let mut out = stderr.lock();

  let mut input = String::new();
  while let Ok(_) = io::stdin().read_line(&mut input) {
    out.write_all(format!("Received: {}", input).as_bytes())?;
  }

  Ok(())
}
