# `script` phase: you usually build, test and generate docs in this phase

set -ex

# PROTIP Always pass `--target $TARGET` to cargo commands, this makes cargo output build artifacts
# to target/$TARGET/{debug,release} which can reduce the number of needed conditionals in the
# `before_deploy`/packaging phase

# Speed up getting multirust-rs by cloning the repo ourselves and not fetching the binaries submodule.
echo Cloning multirust
git clone --single-branch --branch new https://github.com/Diggsey/multirust-rs.git
cd multirust-rs
git checkout c350ddb447b6bd7e431c1a8af796bb5b345b8e8d
cd ..

echo Replacing Cargo.toml to override multirust path
cp ci/Cargo.toml.multirust-override ./Cargo.toml

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

binary=target/$TARGET/release/rust-bisect

# Sanity check the file type.
file $binary

# Print dynamically linked library info.
case $TRAVIS_OS_NAME in
  osx)
    otool -L $binary
    ;;
  linux)
    ldd $binary
    ;;
esac
