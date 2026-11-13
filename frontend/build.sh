#!/usr/bin/env bash
# Build the frontend into public/: vendored Three.js modules and the Elm app.
set -euo pipefail
cd "$(dirname "$0")"
[ -d node_modules/three ] || npm install --no-audit --no-fund
mkdir -p public/vendor/addons/controls
cp node_modules/three/build/three.module.js public/vendor/three.module.js
cp node_modules/three/examples/jsm/controls/OrbitControls.js public/vendor/addons/controls/OrbitControls.js
elm make src/Main.elm --optimize --output=public/main.js
