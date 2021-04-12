BIN=$1
shift 1

set -x

cargo build --release --bin "$BIN"

pushd ~/Projects/ReadOnly/maelstrom

lein run test -w "$BIN" --bin ~/Projects/Rust/maelstrom/target/release/"$BIN" --time-limit 5 --rate 10 "$@"

popd
