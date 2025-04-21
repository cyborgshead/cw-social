# CW-Graph

CW-Graph is a CosmWasm smart contract that implements a flexible, indexed graph database on-chain. It enables the creation and management of semantic relationships (cyberlinks) between entities, supporting complex graph-based data structures directly on the blockchain.

## Overview

The contract provides a comprehensive graph database with the following features:

- **Cyberlink Management**: Create, update, and delete connections between entities
- **Named Cyberlinks**: Support for uniquely identifiable cyberlinks
- **Batch Operations**: Create multiple cyberlinks in a single transaction
- **Rich Querying**: Query cyberlinks by owner, type, source/destination nodes, time ranges, and more
- **Semantic Cores**: Extensible semantic foundation with support for multiple semantic cores
- **Permission Management**: Granular control through admin and executor roles

## Data Model

- **Cyberlink**: A typed connection between two entities (from → to) with an optional value
- **CyberlinkState**: The complete state of a cyberlink including:
  - Type classification
  - Source and destination nodes
  - Associated value/data
  - Ownership information
  - Creation and update timestamps
  - Formatted ID for reference

## Storage & Indexing

The contract implements an optimized storage model with multiple indices for efficient querying:
- Primary key (GID - Graph ID)
- Owner address
- Cyberlink type
- Source node ("from")
- Destination node ("to")
- Formatted ID (FID)
- Composite indices (owner+type, timestamps)


