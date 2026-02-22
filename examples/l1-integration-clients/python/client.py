"""
FlowCortex L1 Python Client

A simple HTTP client for interacting with FlowCortex L1 REST API.
"""

import requests
import base64
import json
from typing import Optional, Dict, Any, List
from dataclasses import dataclass


@dataclass
class BalanceResponse:
    account: str
    token: str
    balance: int


@dataclass
class BlockResponse:
    height: int
    transactions: List[Any]


class FlowCortexL1Client:
    """Client for FlowCortex L1 REST API"""

    def __init__(self, node_url: str = "http://127.0.0.1:3000"):
        self.base_url = node_url
        self.session = requests.Session()
        self.session.headers.update({"Content-Type": "application/json"})

    def create_account(self, account: str) -> Dict[str, Any]:
        """Create a new account"""
        response = self.session.post(
            f"{self.base_url}/account",
            json={"account": account},
        )
        response.raise_for_status()
        return {"success": response.status_code == 201}

    def get_balance(self, account: str, token: str) -> BalanceResponse:
        """Get account balance for a specific token (token should be 'proof' or 'flower')"""
        response = self.session.get(
            f"{self.base_url}/balance/{account}/{token.lower()}"
        )
        response.raise_for_status()
        data = response.json()
        return BalanceResponse(
            account=data["account"],
            token=data["token"],
            balance=data["balance"],
        )

    def mint(
        self,
        caller: str,
        to: str,
        token: str,
        amount: int,
        rw_set: Optional[Dict[str, Any]] = None,
        proof: Optional[Dict[str, Any]] = None,
    ) -> None:
        """Mint tokens to an account (token should be 'proof' or 'flower')"""
        payload = {
            "caller": caller,
            "to": to,
            "token": token.lower(),
            "amount": amount,
        }
        if rw_set:
            payload["rw_set"] = rw_set
        if proof:
            payload["proof"] = proof

        response = self.session.post(
            f"{self.base_url}/mint",
            json=payload,
        )
        response.raise_for_status()

    def transfer(
        self,
        from_account: str,
        to: str,
        token: str,
        amount: int,
        rw_set: Optional[Dict[str, Any]] = None,
        proof: Optional[Dict[str, Any]] = None,
    ) -> None:
        """Transfer tokens between accounts (token should be 'proof' or 'flower')"""
        payload = {
            "from": from_account,
            "to": to,
            "token": token.lower(),
            "amount": amount,
        }
        if rw_set:
            payload["rw_set"] = rw_set
        if proof:
            payload["proof"] = proof

        response = self.session.post(
            f"{self.base_url}/transfer",
            json=payload,
        )
        response.raise_for_status()

    def submit_tx(
        self,
        caller: str,
        pubkey: bytes,
        signature: bytes,
        tx: Dict[str, Any],
    ) -> None:
        """Submit a signed transaction"""
        payload = {
            "caller": caller,
            "pubkey": base64.b64encode(pubkey).decode("utf-8"),
            "signature": base64.b64encode(signature).decode("utf-8"),
            "tx": tx,
        }
        response = self.session.post(
            f"{self.base_url}/tx",
            json=payload,
        )
        response.raise_for_status()

    def get_pool(self) -> Dict[str, Any]:
        """Get pending transaction pool"""
        response = self.session.get(f"{self.base_url}/pool")
        response.raise_for_status()
        return response.json()

    def create_block(self) -> BlockResponse:
        """Create a new block"""
        response = self.session.post(f"{self.base_url}/block")
        response.raise_for_status()
        data = response.json()
        return BlockResponse(
            height=data["height"],
            transactions=data.get("transactions", []),
        )

    def list_blocks(self) -> List[BlockResponse]:
        """List all blocks"""
        response = self.session.get(f"{self.base_url}/blocks")
        response.raise_for_status()
        blocks = response.json()
        return [
            BlockResponse(
                height=block["height"],
                transactions=block.get("transactions", []),
            )
            for block in blocks
        ]

    def get_snapshot(self) -> Dict[str, str]:
        """Get current state snapshot"""
        response = self.session.get(f"{self.base_url}/snapshot")
        response.raise_for_status()
        return response.json()

    def list_anchors(self) -> List[str]:
        """List all anchor IDs"""
        response = self.session.get(f"{self.base_url}/anchors")
        response.raise_for_status()
        data = response.json()
        return data.get("anchors", [])

    def get_anchor(self, anchor_id: str) -> Dict[str, str]:
        """Get a specific anchor by ID"""
        response = self.session.get(f"{self.base_url}/anchor/{anchor_id}")
        response.raise_for_status()
        return response.json()

    def upload_capsule(self, capsule_id: str, code_base64: str) -> Dict[str, Any]:
        """Upload a capsule (wasm/bytecode)"""
        response = self.session.post(
            f"{self.base_url}/capsule",
            json={"id": capsule_id, "code": code_base64},
        )
        response.raise_for_status()
        return response.json()

    def list_capsules(self) -> List[str]:
        """List all capsule IDs"""
        response = self.session.get(f"{self.base_url}/capsule")
        response.raise_for_status()
        data = response.json()
        return data.get("capsules", [])

    def invoke_capsule(self, capsule_id: str, input_base64: str) -> Dict[str, Any]:
        """Invoke a capsule with input data"""
        response = self.session.post(
            f"{self.base_url}/capsule/{capsule_id}/invoke",
            json={"input": input_base64},
        )
        response.raise_for_status()
        return response.json()
