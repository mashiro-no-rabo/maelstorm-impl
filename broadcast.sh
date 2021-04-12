set -x

cargo build --release --bin broadcast

pushd ~/Projects/ReadOnly/maelstrom

lein run test -w broadcast --bin ~/Projects/Rust/maelstrom/target/release/broadcast --time-limit 5 --rate 10 --log-stderr

popd
