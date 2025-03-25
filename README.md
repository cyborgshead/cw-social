# cw-social

## Build
```bash
RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown --lib
```
or
```bash
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.16.0

```

```bash
cargo test
```