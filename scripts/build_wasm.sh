#!/bin/sh

set -ex

# read command line arguments --example <example-name> and --bin <bin-name>
# and set the appropriate cargo command line arguments
# and create the appropriate wasm-bindgen output directory
while [ $# -gt 0 ]; do
  case "$1" in
    --example)
      shift
      cargo_args="--example $1"
      wasm_bindgen_out="./pkg/examples/$1"
      target_wasm="./target/wasm32-unknown-unknown/release/examples/$1.wasm"
      name="$1"
      ;;
    --bin)
      shift
      cargo_args="--bin $1"
      wasm_bindgen_out="./pkg"
      target_wasm="./target/wasm32-unknown-unknown/release/$1.wasm"
      name="$1"
      ;;
    *)
      echo "Unknown argument: $1"
      exit 1
      ;;
  esac
  shift
done

# if no command line arguments are provided, exit
if [ -z "$cargo_args" ]; then
  echo "No example or bin specified"
  exit 1
fi

# A couple of steps are necessary to get this build working which makes it slightly
# nonstandard compared to most other builds.
#
# * First, the Rust standard library needs to be recompiled with atomics
#   enabled. to do that we use Cargo's unstable `-Zbuild-std` feature.
#
# * Next we need to compile everything with the `atomics` and `bulk-memory`
#   features enabled, ensuring that LLVM will generate atomic instructions,
#   shared memory, passive segments, etc.

RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals --cfg=web_sys_unstable_apis' \
  cargo +nightly build $cargo_args --target wasm32-unknown-unknown --release -Z build-std=std,panic_abort

# Note the usage of `--target no-modules` here which is required for passing
# the memory import to each wasm module.
wasm-bindgen \
  $target_wasm \
  --out-dir $wasm_bindgen_out \
  --target no-modules \
  --no-typescript

# copy service worker js file from build-assets to wasm_bindgen_out
cp ./build-assets/service-worker.js $wasm_bindgen_out

# copy css file
cp ./build-assets/style.css $wasm_bindgen_out

# copy index.html file
cp ./build-assets/index.html $wasm_bindgen_out

# rename the [$name]_bg.wasm file to bg.wasm (make sure to use the name from the command line arguments)
mv $wasm_bindgen_out/$name"_bg.wasm" $wasm_bindgen_out/bg.wasm

# rename the script file to script.js
mv $wasm_bindgen_out/$name.js $wasm_bindgen_out/script.js

# use cargo server to serve the wasm-bindgen output directory
cargo server --port 8080 --path $wasm_bindgen_out --open
