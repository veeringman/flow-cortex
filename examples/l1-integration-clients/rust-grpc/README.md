# FlowCortex L1 gRPC Client Example (Rust)

This example demonstrates how to build a Rust gRPC client to interact with the FlowCortex L1 node.

## Building

```bash
cargo build --release
```

## Running

Make sure the FlowCortex L1 node is running on `127.0.0.1:50051`:

```bash
cargo run --release
```

## Example Methods

The client demonstrates:
- **Get Balance**: Query account token balances
- **List Blocks**: Retrieve all blocks in the chain
- **List Anchors**: List all anchor proofs
- **Get Snapshot**: Get the current state snapshot root
- **Upload Capsule**: Submit wasm/bytecode capsules
- **List Capsules**: List all uploaded capsules

## Integration

To use in your own application:

1. Copy the `proto/l1.proto` file to your project
2. Add `tonic` and `prost` to your `Cargo.toml`
3. Create a `build.rs` to compile the proto file
4. Create a client and connect:

```rust
use tonic::transport::Channel;

let channel = Channel::from_static("http://127.0.0.1:50051")
    .connect()
    .await?;
let mut client = L1Client::new(channel);

let response = client.get_balance(
    tonic::Request::new(BalanceRequest {
        account: "alice".to_string(),
        token: "Proof".to_string(),
    })
).await?;
```
