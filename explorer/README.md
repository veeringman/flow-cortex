# flowcortex-explorer

A minimal web UI for inspecting the L1 node. It serves a static HTML page that currently just says `Hello` but will eventually provide account/balance queries, block lists, etc.

## Running

By default the server listens on `0.0.0.0:4000`. You can override the bind address with the `BIND_ADDR` environment variable:

```sh
# listen on all interfaces (public)
BIND_ADDR=0.0.0.0:4000 cargo run --manifest-path explorer/Cargo.toml

# listen locally only
BIND_ADDR=127.0.0.1:4000 cargo run --manifest-path explorer/Cargo.toml
```

The `scripts/run_servers.sh` helper will start both the L1 node and explorer with sensible defaults.
