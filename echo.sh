set -x

cargo build --release --bin echo

pushd ~/Projects/ReadOnly/maelstrom

lein run test -w echo --bin ~/Projects/Rust/maelstrom/target/release/echo --nodes n1 --time-limit 2 --log-stderr

popd
