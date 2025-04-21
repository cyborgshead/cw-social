# CW Social

CW Social is a decentralized social graph protocol built on CosmWasm for Cyber. It provides a foundation for social applications by managing user relationships, content, and interactions in a blockchain environment.

## Features

- **Decentralized Social Graph**: Create and manage social connections between users
- **Decentralized Knowledge Graph**: Create and manage personal and global knowledge graph
- **Customizable Interactions**: Support for various social actions like follows, likes, and comments
- **Permissionless**: Open protocol that any application can build upon
- **Composable**: Designed to work with other CosmWasm contracts and ecosystems

## Overview

The CW Social protocol enables developers to build social applications without centralized control of user data. Users own their connections and content, while applications can provide unique interfaces and experiences on top of the shared social graph.

## Contracts
- [cw-graph](./contracts/cw-graph/README.md): The main contract that handles the knowledge graph functionality.

## Build

```bash
cargo wasm
```

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
and
```bash
cargo test
```

```bash
cargo run schema
```