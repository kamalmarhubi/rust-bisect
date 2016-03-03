# `script` phase: you usually build, test and generate docs in this phase

set -ex

# PROTIP Always pass `--target $TARGET` to cargo commands, this makes cargo output build artifacts
# to target/$TARGET/{debug,release} which can reduce the number of needed conditionals in the
# `before_deploy`/packaging phase

# Disable doctests when cross-compiling as there appears to be an issue.
host=$(rustc -vV | grep host | awk '{print $2}')
if [ $TARGET != $host ]; then
  cat >> Cargo.toml <<EOF
[lib]
doctest = false
EOF
fi

cargo build --target $TARGET --verbose
cargo run --target $TARGET --example rust-issue-30123
cargo test --target $TARGET
cargo build --target $TARGET --release

# sanity check the file type
file target/$TARGET/release/rust-bisect
