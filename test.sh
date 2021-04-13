BIN=$1
shift 1

set -ex
cargo build --release --bin "$BIN"
set +e

pushd ~/Projects/ReadOnly/maelstrom

lein run test -w "$BIN" --bin ~/Projects/Rust/maelstrom/target/release/"$BIN" --time-limit 5 "$@"

popd
