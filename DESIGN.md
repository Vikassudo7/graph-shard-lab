# GraphShard Lab Design

## Goal

GraphShard Lab compares different ways of placing graph data across logical shards.

The main trade-off is:

- balanced shards;
- fewer cross-shard graph traversals.

## Graph model

Users are graph nodes.

A directed `FOLLOWS` relationship is an edge:

```text
User 1 → User 2
