# FlowCortex Integration Guide - Wallet Team

**Version:** 1.0  
**Date:** February 23, 2026  
**Contact:** flowcortex-integrations@example.com

---

## Overview

Your wallet application displays real-time settlement status to end users. FlowCortex provides event streams and query APIs to show users that their settlements are "provably authorized" with cryptographic guarantees.

---

## Your Role

1. User initiates settlement in your wallet UI
2. Backend processes through FortressDigital + FlowCortex
3. **→ You display real-time settlement status to user**
4. **→ You show "Provably Authorized ✅" badge when verified**

---

## API Endpoints

**Base URL:**
- Development: `https://dev-l1.flowcortex.example.com`
- Production: `https://l1.flowcortex.example.com`

**UI-Relevant Endpoints:**
- Query status: `GET /api/proof_status/{commitment_hash}`
- Event stream: `WS /api/events/subscribe`
- Dashboard stats: `GET /api/demo/dashboard/stats`

---

## Integration Example

### 1. Display Settlement Status

```javascript
// React component example
import { useEffect, useState } from 'react';
import { FlowCortexClient } from '@flowcortex/sdk';

function SettlementStatus({ commitmentHash }) {
  const [status, setStatus] = useState('pending');
  const [verified, setVerified] = useState(false);
  
  useEffect(() => {
    const client = new FlowCortexClient({
      apiKey: process.env.FLOWCORTEX_API_KEY
    });
    
    // Poll for status
    const interval = setInterval(async () => {
      const result = await client.getProofStatus(commitmentHash);
      
      if (result.verified) {
        setStatus('verified');
        setVerified(true);
        clearInterval(interval);
      }
    }, 2000); // Poll every 2 seconds
    
    return () => clearInterval(interval);
  }, [commitmentHash]);
  
  return (
    <div className="settlement-status">
      {status === 'pending' && (
        <div className="status-pending">
          ⏳ Verifying authorization...
        </div>
      )}
      
      {status === 'verified' && (
        <div className="status-verified">
          ✅ Provably Authorized
          <span className="badge">Zero-Knowledge Verified</span>
        </div>
      )}
    </div>
  );
}
```

### 2. Real-Time Event UI (WebSocket)

```javascript
import { useEffect, useState } from 'react';

function SettlementTracker({ commitmentHash }) {
  const [events, setEvents] = useState([]);
  const [currentStep, setCurrentStep] = useState(1);
  
  useEffect(() => {
    const ws = new WebSocket(
      `wss://l1.flowcortex.example.com/api/events/subscribe`
    );
    
    ws.onopen = () => {
      ws.send(JSON.stringify({
        action: 'subscribe',
        commitment_hash: commitmentHash
      }));
    };
    
    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      
      setEvents(prev => [...prev, data]);
      
      if (data.type === 'CommitmentAnchored') {
        setCurrentStep(2);
      } else if (data.type === 'ProofVerified') {
        setCurrentStep(3);
      }
    };
    
    return () => ws.close();
  }, [commitmentHash]);
  
  return (
    <div className="settlement-tracker">
      <StepIndicator step={1} current={currentStep} 
        label="Authorization Request" />
      <StepIndicator step={2} current={currentStep} 
        label="Commitment Anchored" />
      <StepIndicator step={3} current={currentStep} 
        label="Proof Verified" />
    </div>
  );
}
```

---

## Status Response Format

```json
{
  "commitment_hash": "a1b2c3...",
  "verified": true,
  "proof": {
    "proof_hash": "b2c3d4...",
    "verification_block": 12346,
    "verified_at": 1708704010
  }
}
```

**Status States:**
- `verified: false` → Show "⏳ Pending Verification"
- `verified: true` → Show "✅ Provably Authorized"
- No proof found → Show "Verifying..."

---

## Event Types for UI

### CommitmentAnchored

```json
{
  "type": "CommitmentAnchored",
  "commitment_hash": "a1b2c3...",
  "block_height": 12345,
  "timestamp": 1708704001
}
```

**UI Action:** Show "Authorization recorded on FlowCortex" 

### ProofVerified

```json
{
  "type": "ProofVerified",
  "commitment_hash": "a1b2c3...",
  "proof_hash": "b2c3d4...",
  "verification_block": 12346,
  "verified_at": 1708704010
}
```

**UI Action:** Show "✅ Provably Authorized" badge

### ProofVerificationFailed

```json
{
  "type": "ProofVerificationFailed",
  "commitment_hash": "a1b2c3...",
  "error_reason": "PROOF_INVALID",
  "failed_at": 1708704010
}
```

**UI Action:** Show "❌ Authorization Failed" and block settlement

---

## UI Components

### Status Badge

```html
<!-- Verified -->
<div class="badge verified">
  <span class="icon">✅</span>
  <span class="text">Provably Authorized</span>
  <span class="detail">Zero-Knowledge Verified</span>
</div>

<!-- Pending -->
<div class="badge pending">
  <span class="icon">⏳</span>
  <span class="text">Verifying...</span>
</div>

<!-- Failed -->
<div class="badge failed">
  <span class="icon">❌</span>
  <span class="text">Verification Failed</span>
</div>
```

### Timeline View

```
Step 1: Authorization Request          ✅ Complete
  └─ FortressDigital policy decision
  
Step 2: Commitment Anchored            ✅ Complete
  └─ Block Height: 12345
  
Step 3: Proof Generated                ⏳ In Progress
  └─ ProofCortex generating STARK proof
  
Step 4: Proof Verified                 ⏳ Pending
  └─ Awaiting verification
```

---

## Dashboard API (Optional)

Get aggregated stats for user dashboard:

```javascript
const response = await fetch(
  'https://l1.flowcortex.example.com/api/demo/dashboard/stats',
  {
    headers: { 'Authorization': 'Bearer YOUR_KEY' }
  }
);

const stats = await response.json();
// {
//   "total_settlements": 156,
//   "verified_settlements": 154,
//   "pending_verifications": 2,
//   "average_verification_time_ms": 2340
// }
```

**Display in UI:**
```
Settlement Statistics
━━━━━━━━━━━━━━━━━━━━━
Total Processed:    156
Successfully Verified: 154 (98.7%)
Pending:            2
Avg. Time:          2.3 seconds
```

---

## Best Practices

1. **Polling vs WebSocket:**
   - Use WebSocket for real-time updates (better UX)
   - Fall back to polling if WebSocket unavailable
   - Poll every 2-5 seconds, not more frequently

2. **User Communication:**
   - Use clear language: "Verifying authorization..." not "Waiting for proof"
   - Show progress indicators for steps
   - Explain what "Provably Authorized" means in simple terms

3. **Error Handling:**
   - If verification fails, show clear message
   - Provide user action (contact support, try again)
   - Never hide verification failures

4. **Performance:**
   - Cache verification status (it's immutable once verified)
   - Debounce status checks
   - Use CDN for badge assets

---

## Mobile SDK

**React Native:**
```javascript
import { FlowCortexClient } from '@flowcortex/react-native-sdk';

const client = new FlowCortexClient({
  apiKey: FLOWCORTEX_API_KEY,
  environment: 'production'
});

// Subscribe to status changes
client.subscribeToCommitment(commitmentHash, (event) => {
  if (event.type === 'ProofVerified') {
    showNotification('Settlement Approved ✅');
  }
});
```

**iOS (Swift):**
```swift
import FlowCortexSDK

let client = FlowCortexClient(apiKey: "YOUR_KEY")

client.getProofStatus(commitmentHash: hash) { result in
    switch result {
    case .success(let status):
        if status.verified {
            self.showVerifiedBadge()
        }
    case .failure(let error):
        self.showError(error)
    }
}
```

**Android (Kotlin):**
```kotlin
import com.flowcortex.sdk.FlowCortexClient

val client = FlowCortexClient(apiKey = "YOUR_KEY")

client.getProofStatus(commitmentHash) { result ->
    result.onSuccess { status ->
        if (status.verified) {
            showVerifiedBadge()
        }
    }
}
```

---

## Testing

**Mock Data (Development):**
```javascript
// For development/testing without real FlowCortex
const mockStatus = {
  verified: true,
  proof: {
    verification_block: 12346,
    verified_at: Date.now() / 1000
  }
};

// Use in non-production environments
if (process.env.NODE_ENV === 'development') {
  return mockStatus;
}
```

---

## Rate Limits

- Development: 10 requests/second
- Production: 100 requests/second per API key
- WebSocket: 1000 concurrent connections

---

## Support

- UI/UX Questions: integrations@flowcortex.example.com
- SDK Issues: sdk-support@flowcortex.example.com
- Production: support@flowcortex.example.com (24/7)
