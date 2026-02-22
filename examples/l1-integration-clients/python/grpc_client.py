"""
FlowCortex L1 gRPC Python Client

A gRPC client for interacting with FlowCortex L1 gRPC API.
Requires grpcio and grpcio-tools to be installed.

Installation:
    pip install grpcio grpcio-tools

Generate gRPC code:
    python -m grpc_tools.protoc -I proto --python_out=. --grpc_python_out=. proto/l1.proto
"""

import grpc
import base64
import sys
from typing import Optional

# These imports assume you've generated the gRPC code
# from protos. Install grpcio-tools and run:
# python -m grpc_tools.protoc -I proto --python_out=. --grpc_python_out=. proto/l1.proto
try:
    from l1_pb2 import (
        Empty,
        BalanceRequest,
        BalanceResponse,
        AnchorRequest,
        AnchorListResponse,
        CapsuleUploadRequest,
        CapsuleListResponse,
        CapsuleInvokeRequest,
    )
    from l1_pb2_grpc import L1Stub
except ImportError:
    print("Error: gRPC protobuf files not found.")
    print(
        "Please generate them with: python -m grpc_tools.protoc -I proto --python_out=. --grpc_python_out=. proto/l1.proto"
    )
    sys.exit(1)


class FlowCortexL1GRPCClient:
    """gRPC client for FlowCortex L1"""

    def __init__(self, host: str = "127.0.0.1", port: int = 50051):
        channel = grpc.aio.secure_channel(f"{host}:{port}", grpc.ssl_channel_credentials())
        self.stub = L1Stub(channel)

    async def get_balance(self, account: str, token: str) -> BalanceResponse:
        """Get account balance"""
        request = BalanceRequest(account=account, token=token)
        return await self.stub.GetBalance(request)

    async def list_blocks(self):
        """List all blocks"""
        return await self.stub.ListBlocks(Empty())

    async def list_anchors(self) -> AnchorListResponse:
        """List all anchors"""
        return await self.stub.ListAnchors(Empty())

    async def get_anchor(self, anchor_id: str):
        """Get a specific anchor"""
        request = AnchorRequest(id=anchor_id)
        return await self.stub.GetAnchor(request)

    async def snapshot(self):
        """Get current snapshot"""
        return await self.stub.Snapshot(Empty())

    async def upload_capsule(self, capsule_id: str, code: bytes):
        """Upload a capsule"""
        request = CapsuleUploadRequest(id=capsule_id, code=code)
        return await self.stub.UploadCapsule(request)

    async def list_capsules(self) -> CapsuleListResponse:
        """List all capsules"""
        return await self.stub.ListCapsules(Empty())

    async def invoke_capsule(self, capsule_id: str, input_data: bytes):
        """Invoke a capsule"""
        request = CapsuleInvokeRequest(id=capsule_id, input=input_data)
        return await self.stub.InvokeCapsule(request)


# Example usage (for synchronous wrapper, use grpc channel instead)
async def main():
    """Example usage of gRPC client"""
    client = FlowCortexL1GRPCClient()

    print("=== FlowCortex L1 gRPC Python Client Examples ===\n")

    try:
        # Example: Get balance
        print("1. Getting balance...")
        balance = await client.get_balance("admin", "Proof")
        print(f"   Account: {balance.account}")
        print(f"   Token: {balance.token}")
        print(f"   Balance: {balance.balance}\n")

        # Example: List blocks
        print("2. Listing blocks...")
        blocks_response = await client.list_blocks()
        print(f"   Total blocks: {len(blocks_response.blocks)}\n")

        # Example: List anchors
        print("3. Listing anchors...")
        anchors = await client.list_anchors()
        print(f"   Total anchors: {len(anchors.ids)}\n")

        # Example: Get snapshot
        print("4. Getting snapshot...")
        snapshot = await client.snapshot()
        print(f"   Root: {snapshot.root}\n")

        # Example: Upload capsule
        print("5. Uploading capsule...")
        capsule_response = await client.upload_capsule("test_capsule", b"sample code")
        print(f"   Success: {capsule_response.success}\n")

        # Example: List capsules
        print("6. Listing capsules...")
        capsules = await client.list_capsules()
        print(f"   Total capsules: {len(capsules.capsules)}\n")

        print("=== Examples completed! ===")

    except grpc.RpcError as e:
        print(f"gRPC Error: {e.code()}: {e.details()}")
        sys.exit(1)


if __name__ == "__main__":
    import asyncio

    asyncio.run(main())
