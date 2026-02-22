# FlowCortex L1 Python Client

Complete Python examples for integrating with FlowCortex L1 REST and gRPC APIs.

## Installation

### REST Client (Recommended)

```bash
pip install -r requirements.txt
```

### gRPC Client (Optional)

If you want to use the gRPC client, additionally generate the protobuf files:

```bash
python -m grpc_tools.protoc -I . --python_out=. --grpc_python_out=. l1.proto
```

## Usage

### REST API Example

```bash
python example.py
```

This demonstrates:
- Creating accounts
- Getting token balances
- Minting and transferring tokens
- Managing blocks
- Uploading and invoking capsules
- Working with anchors

### gRPC Example

```bash
python grpc_client.py
```

This demonstrates gRPC-based operations for:
- Getting balances
- Listing blocks
- Managing anchors
- Snapshot queries
- Capsule management

## REST Client Class

```python
from client import FlowCortexL1Client

# Initialize client
client = FlowCortexL1Client('http://127.0.0.1:3000')

# Get balance
balance = client.get_balance('alice', 'Proof')
print(f"Balance: {balance.balance}")

# Transfer tokens
client.transfer(
    from_account='alice',
    to='bob',
    token='Proof',
    amount=100
)

# Upload capsule
import base64
code = base64.b64encode(b"wasm code").decode()
client.upload_capsule('my_capsule', code)

# List blocks
blocks = client.list_blocks()
print(f"Total blocks: {len(blocks)}")
```

## gRPC Client Class

```python
import asyncio
from grpc_client import FlowCortexL1GRPCClient

async def main():
    client = FlowCortexL1GRPCClient('127.0.0.1', 50051)
    
    # Get balance
    balance = await client.get_balance('admin', 'Proof')
    print(f"Balance: {balance.balance}")
    
    # List blocks
    blocks = await client.list_blocks()
    print(f"Total blocks: {len(blocks.blocks)}")

asyncio.run(main())
```

## API Methods (REST Client)

### Account Management
- `create_account(account: str) -> Dict`
- `get_balance(account: str, token: str) -> BalanceResponse`

### Transactions
- `mint(caller, to, token, amount, rw_set=None, proof=None) -> None`
- `transfer(from_account, to, token, amount, rw_set=None, proof=None) -> None`
- `submit_tx(caller, pubkey, signature, tx) -> None`
- `get_pool() -> Dict`

### Blocks
- `create_block() -> BlockResponse`
- `list_blocks() -> List[BlockResponse]`
- `get_snapshot() -> Dict`

### Capsules
- `upload_capsule(capsule_id, code_base64) -> Dict`
- `list_capsules() -> List[str]`
- `invoke_capsule(capsule_id, input_base64) -> Dict`

### Anchors
- `list_anchors() -> List[str]`
- `get_anchor(anchor_id) -> Dict`

## Error Handling

```python
try:
    client.transfer(from_account='alice', to='bob', token='Proof', amount=100)
except requests.exceptions.RequestException as e:
    print(f"Error: {e.response.text if e.response else e}")
```

## Advanced Usage

### With custom read-write sets and proofs

```python
client.transfer(
    from_account='alice',
    to='bob',
    token='Proof',
    amount=100,
    rw_set={'...': '...'},
    proof={'...': '...'}
)
```

### Batch operations

```python
for i in range(10):
    client.transfer(from_account='admin', to=f'user_{i}', token='Proof', amount=10)
    blocks = client.list_blocks()
    print(f"Created {len(blocks)} blocks")
```
