BIN=$1
shift 1

set -ex

if [[ -z "$FEATURES" ]]; then
  cargo build --release --bin "$BIN"
else
  cargo build --release --bin "$BIN" --features "$FEATURES"
fi

set +e

pushd ~/Projects/ReadOnly/maelstrom

lein run test -w "$BIN" --bin ~/Projects/Rust/maelstrom/target/release/"$BIN" --time-limit 5 "$@"

popd
