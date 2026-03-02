# FlowCortex L1 TypeScript/JavaScript Client

Complete TypeScript and JavaScript examples for integrating with FlowCortex L1 REST API.

## Installation

```bash
npm install
npm run build
```

## Usage

### Node.js Example

```bash
npm run example:node
```

This runs `examples/node-example.ts` which demonstrates:
- Creating accounts
- Getting balances
- Minting and transferring tokens
- Submitting transactions
- Managing blocks
- Uploading and invoking capsules
- Working with anchors

### Browser Example

```bash
npm run example:browser
```

Then open `http://192.168.29.78:8080/src/examples/browser-example.html` in your browser.

The browser client provides an interactive UI for:
- Account management
- Token transfers and minting
- Block creation and querying
- Capsule management
- Anchor and pool queries

## API Client Class

Import and use the `FlowCortexL1Client` in your project:

```typescript
import { FlowCortexL1Client } from './client';

const client = new FlowCortexL1Client('http://127.0.0.1:3000');

// Get balance
const balance = await client.getBalance('alice', 'Proof');
console.log(`Balance: ${balance.balance}`);

// Transfer tokens
await client.transfer({
  from: 'alice',
  to: 'bob',
  token: 'Proof',
  amount: 100,
});

// List blocks
const blocks = await client.listBlocks();
console.log(`Total blocks: ${blocks.length}`);

// Upload capsule
await client.uploadCapsule('my_capsule', base64EncodedCode);

// Invoke capsule
const output = await client.invokeCapsule('my_capsule', base64EncodedInput);
```

## API Methods

### Account Management
- `createAccount(account: string): Promise<{ success: boolean }>`
- `getBalance(account: string, token: 'Proof' | 'FloweR'): Promise<BalanceResponse>`

### Transactions
- `mint(params): Promise<void>`
- `transfer(params): Promise<void>`
- `submitTx(params): Promise<void>`
- `getPool(): Promise<PoolResponse>`

### Blocks
- `createBlock(): Promise<BlockResponse>`
- `listBlocks(): Promise<BlockResponse[]>`
- `getSnapshot(): Promise<SnapshotResponse>`

### Capsules
- `uploadCapsule(id: string, codeBase64: string): Promise<CapsuleResponse>`
- `listCapsules(): Promise<CapsuleListResponse>`
- `invokeCapsule(id: string, inputBase64: string): Promise<CapsuleInvokeResponse>`

### Anchors
- `listAnchors(): Promise<AnchorListResponse>`
- `getAnchor(id: string): Promise<AnchorResponse>`

## Configuration

The client connects to `http://127.0.0.1:3000` by default. Override this:

```typescript
const client = new FlowCortexL1Client('http://your-node:3000');
```

## Error Handling

All methods throw `AxiosError` on failure. Handle appropriately:

```typescript
try {
  await client.transfer({ ... });
} catch (error) {
  console.error('Transfer failed:', error.response?.data || error.message);
}
```
