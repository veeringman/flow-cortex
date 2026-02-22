#!/usr/bin/env python3
"""
Example usage of FlowCortex L1 Python client

Demonstrates all major operations:
- Account management
- Token transfers and minting
- Block creation and querying
- Capsule management
- Anchor queries
"""

import base64
import sys
from client import FlowCortexL1Client


def main():
    client = FlowCortexL1Client("http://127.0.0.1:3000")

    print("=== FlowCortex L1 Python Client Examples ===\n")

    try:
        # Example 1: Create accounts
        print("1. Creating accounts...")
        client.create_account("alice")
        client.create_account("bob")
        print("   ✓ Accounts created\n")

        # Example 2: Get balance
        print("2. Getting admin balance...")
        balance = client.get_balance("admin", "Proof")
        print(f"   Account: {balance.account}")
        print(f"   Token: {balance.token}")
        print(f"   Balance: {balance.balance}\n")

        # Example 3: Mint tokens
        print("3. Minting tokens to alice...")
        client.mint(
            caller="admin",
            to="alice",
            token="Proof",
            amount=1000,
        )
        print("   ✓ Tokens minted\n")

        # Example 4: Check minted balance
        alice_balance = client.get_balance("alice", "Proof")
        print(f"4. Alice's new balance: {alice_balance.balance}\n")

        # Example 5: Transfer tokens
        print("5. Transferring tokens from alice to bob...")
        client.transfer(
            from_account="alice",
            to="bob",
            token="Proof",
            amount=100,
        )
        print("   ✓ Tokens transferred\n")

        # Example 6: Get pending pool
        print("6. Getting pending transaction pool...")
        pool = client.get_pool()
        print(f"   Pool: {pool}\n")

        # Example 7: List blocks
        print("7. Listing blocks...")
        blocks = client.list_blocks()
        print(f"   Total blocks: {len(blocks)}")
        for i, block in enumerate(blocks[:3]):
            print(f"     Block {i}: height={block.height}, txs={len(block.transactions)}")
        print("")

        # Example 8: Get snapshot
        print("8. Getting snapshot...")
        snapshot = client.get_snapshot()
        print(f"   Root: {snapshot.get('root', 'N/A')}\n")

        # Example 9: Create block
        print("9. Creating new block...")
        new_block = client.create_block()
        print(f"   New block: height={new_block.height}\n")

        # Example 10: Upload capsule
        print("10. Uploading capsule...")
        sample_code = base64.b64encode(b"sample capsule code").decode("utf-8")
        capsule_response = client.upload_capsule("my_capsule_1", sample_code)
        print(f"   Success: {capsule_response.get('success', False)}\n")

        # Example 11: List capsules
        print("11. Listing capsules...")
        capsules = client.list_capsules()
        print(f"   Total capsules: {len(capsules)}")
        for i, capsule_id in enumerate(capsules):
            print(f"     Capsule {i}: {capsule_id}")
        print("")

        # Example 12: Invoke capsule
        print("12. Invoking capsule...")
        input_data = base64.b64encode(b"test input").decode("utf-8")
        try:
            invoke_response = client.invoke_capsule("my_capsule_1", input_data)
            print(f"   Output: {invoke_response.get('output', 'N/A')}\n")
        except Exception as e:
            print(f"   Note: {e}\n")

        # Example 13: List anchors
        print("13. Listing anchors...")
        anchors = client.list_anchors()
        print(f"   Total anchors: {len(anchors)}")
        for i, anchor_id in enumerate(anchors[:3]):
            print(f"     Anchor {i}: {anchor_id}")
        print("")

        print("=== All examples completed successfully! ===")

    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
