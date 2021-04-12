BIN=$1
shift 1
TIME="${TIME:-5}"
RATE="${RATE:-10}"

set -x

cargo build --release --bin "$BIN"

pushd ~/Projects/ReadOnly/maelstrom

lein run test -w "$BIN" --bin ~/Projects/Rust/maelstrom/target/release/"$BIN" --time-limit "$TIME" --rate "$RATE" "$@"

popd
