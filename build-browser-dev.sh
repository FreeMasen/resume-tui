#! /bin/bash
echo "creating dummy preview file" \
    && echo "# Live Example" > ./live-preview.md \
    && echo "fetching oranda" \
    && curl --proto '=https' --tlsv1.2 -LsSf https://github.com/axodotdev/oranda/releases/latest/download/oranda-installer.sh | sh \
    && echo "building main site" \
    && oranda build \
    && echo "building browser" \
    && pushd ./browser \
    && echo "wasm-pack build" \
    && wasm-pack build --target web --no-pack --release \
    && popd \
    && echo "rm .gitignore" \
    && rm ./browser/pkg/.gitignore || true \
    && echo "rm live-preview.md" \
    && rm ./public/live-preview.md || true \
    && echo "rm making dir" \
    && mkdir -p ./public/live-preview \
    && echo "cp wasm pack content" \
    && cp -r ./browser/pkg/* ./public/live-preview/ \
    && echo "cp index.html" \
    && cp -r ./browser/index.html ./public/live-preview/
