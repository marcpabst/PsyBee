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
      ;;
    --bin)
      shift
      cargo_args="--bin $1"
      wasm_bindgen_out="./pkg"
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
  target/wasm32-unknown-unknown/release/examples/image.wasm \
  --out-dir $wasm_bindgen_out \
  --target no-modules

# create service worker js file
cat > $wasm_bindgen_out/service-worker.js <<EOF
/*! coi-serviceworker v0.1.7 - Guido Zuidhof and contributors, licensed under MIT */
let coepCredentialless=!1;"undefined"==typeof window?(self.addEventListener("install",(()=>self.skipWaiting())),self.addEventListener("activate",(e=>e.waitUntil(self.clients.claim()))),self.addEventListener("message",(e=>{e.data&&("deregister"===e.data.type?self.registration.unregister().then((()=>self.clients.matchAll())).then((e=>{e.forEach((e=>e.navigate(e.url)))})):"coepCredentialless"===e.data.type&&(coepCredentialless=e.data.value))})),self.addEventListener("fetch",(function(e){const o=e.request;if("only-if-cached"===o.cache&&"same-origin"!==o.mode)return;const s=coepCredentialless&&"no-cors"===o.mode?new Request(o,{credentials:"omit"}):o;e.respondWith(fetch(s).then((e=>{if(0===e.status)return e;const o=new Headers(e.headers);return o.set("Cross-Origin-Embedder-Policy",coepCredentialless?"credentialless":"require-corp"),coepCredentialless||o.set("Cross-Origin-Resource-Policy","cross-origin"),o.set("Cross-Origin-Opener-Policy","same-origin"),new Response(e.body,{status:e.status,statusText:e.statusText,headers:o})})).catch((e=>console.error(e))))}))):(()=>{const e=window.sessionStorage.getItem("coiReloadedBySelf");window.sessionStorage.removeItem("coiReloadedBySelf");const o="coepdegrade"==e,s={shouldRegister:()=>!e,shouldDeregister:()=>!1,coepCredentialless:()=>!0,coepDegrade:()=>!0,doReload:()=>window.location.reload(),quiet:!1,...window.coi},r=navigator,t=r.serviceWorker&&r.serviceWorker.controller;t&&!window.crossOriginIsolated&&window.sessionStorage.setItem("coiCoepHasFailed","true");const i=window.sessionStorage.getItem("coiCoepHasFailed");if(t){const e=s.coepDegrade()&&!(o||window.crossOriginIsolated);r.serviceWorker.controller.postMessage({type:"coepCredentialless",value:!(e||i&&s.coepDegrade())&&s.coepCredentialless()}),e&&(!s.quiet&&console.log("Reloading page to degrade COEP."),window.sessionStorage.setItem("coiReloadedBySelf","coepdegrade"),s.doReload("coepdegrade")),s.shouldDeregister()&&r.serviceWorker.controller.postMessage({type:"deregister"})}!1===window.crossOriginIsolated&&s.shouldRegister()&&(window.isSecureContext?r.serviceWorker?r.serviceWorker.register(window.document.currentScript.src).then((e=>{!s.quiet&&console.log("COOP/COEP Service Worker registered",e.scope),e.addEventListener("updatefound",(()=>{!s.quiet&&console.log("Reloading page to make use of updated COOP/COEP Service Worker."),window.sessionStorage.setItem("coiReloadedBySelf","updatefound"),s.doReload()})),e.active&&!r.serviceWorker.controller&&(!s.quiet&&console.log("Reloading page to make use of COOP/COEP Service Worker."),window.sessionStorage.setItem("coiReloadedBySelf","notcontrolling"),s.doReload())}),(e=>{!s.quiet&&console.error("COOP/COEP Service Worker failed to register:",e)})):!s.quiet&&console.error("COOP/COEP Service Worker not registered, perhaps due to private mode."):!s.quiet&&console.log("COOP/COEP Service Worker not registered, a secure context is required."))})();
EOF

# create an index.html file
cat > $wasm_bindgen_out/index.html <<EOF
<!DOCTYPE html>

<head>
  <meta charset="UTF-8" />
  <script src="./service-worker.js"></script>
  <style>
  * {
      margin: 0;
      padding: 0;
    }
    html, body { width: 100%; height: 100%; }
    .hidden {
      display: none;
    }
    canvas {
      image-rendering: pixelated;
      image-rendering: crisp-edges;
  }
  </style>
</head>

<body>
  <script src="./image.js"></script>
  <script type="text/javascript">
  document.onclick = function() {
    wasm_bindgen('./image_bg.wasm').then((wasm) => {
      //wasm.run();
    });
  }
  </script>
</body>
</html>
EOF