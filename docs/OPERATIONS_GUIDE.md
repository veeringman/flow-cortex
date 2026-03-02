# FlowCortex Operations Guide

**Version:** 1.0  
**Date:** February 23, 2026  

---

## Quick Start

### Local Development Setup

```bash
# Clone repository
git clone https://github.com/flowcortex/flow-cortex
cd flow-cortex/flowcortex-l1

# Run development node
cargo run

# In another terminal, run tests
cargo test

# Start explorer
cd ../explorer
cargo run
```

Access:
- API: http://192.168.29.78:3000
- Explorer: http://192.168.29.78:8080

---

## Environment Variables

```bash
# Required
FLOWCORTEX_API_KEY=your_key_here
FLOWCORTEX_ENVIRONMENT=development|production

# Optional
FLOWCORTEX_LOG_LEVEL=info
FLOWCORTEX_RATE_LIMIT=100
```

---

## Deployment

### Docker Deployment

```bash
# Build image
docker build -t flowcortex-l1:latest .

# Run container
docker run -p 3000:3000 \
  -e FLOWCORTEX_API_KEY=your_key \
  flowcortex-l1:latest
```

### Production Deployment

```bash
# Build release
cargo build --release

# Run with production config
./target/release/flowcortex-l1 \
  --config config/prod/config.toml \
  --log-level info
```

---

## Monitoring

### Health Check

```bash
curl http://192.168.29.78:3000/health

# Expected response:
# {"status": "healthy", "block_height": 12345}
```

### Metrics

Available at `/metrics`:
- `flowcortex_commitments_total`
- `flowcortex_proofs_verified_total`
- `flowcortex_api_requests_total`
- `flowcortex_api_latency_seconds`

---

## Backup & Recovery

### Create Snapshot

```bash
# Backup ledger state
curl -X POST http://192.168.29.78:3000/admin/snapshot \
  -H "Authorization: Bearer ADMIN_KEY"

# Saved to: /var/lib/flowcortex/snapshots/snapshot_TIMESTAMP.json
```

### Restore from Snapshot

```bash
./flowcortex-l1 --restore /path/to/snapshot.json
```

---

## Troubleshooting

### Common Issues

**Issue: API not responding**
```bash
# Check if process is running
ps aux | grep flowcortex-l1

# Check logs
tail -f /var/log/flowcortex/flowcortex-l1.log

# Restart service
systemctl restart flowcortex-l1
```

**Issue: High latency**
```bash
# Check current load
curl http://192.168.29.78:3000/metrics | grep latency

# Increase rate limits in config
```

**Issue: Verification failures**
```bash
# Check capsule registry
curl http://192.168.29.78:3000/admin/capsules

# Verify capsule version
curl http://192.168.29.78:3000/admin/capsules/verifier_v1
```

---

## Performance Tuning

### Expected Performance
- Commitment anchoring: < 50ms p99
- Proof verification: < 100ms p99
- Query operations: < 20ms p99
- Throughput: 1000 req/sec sustained

### Tuning Parameters

```toml
# config.toml
[performance]
max_concurrent_requests = 1000
connection_pool_size = 100
query_cache_ttl = 300  # seconds
```

---

## Security

### API Key Rotation

```bash
# Generate new key
curl -X POST http://192.168.29.78:3000/admin/keys/generate \
  -H "Authorization: Bearer ADMIN_KEY"

# Revoke old key
curl -X DELETE http://192.168.29.78:3000/admin/keys/{key_id} \
  -H "Authorization: Bearer ADMIN_KEY"
```

### Access Logs

```bash
# View recent API access
tail -f /var/log/flowcortex/access.log

# Filter by IP
grep "192.168.1.100" /var/log/flowcortex/access.log
```

---

## Support

- Production Issues: support@flowcortex.example.com (24/7)
- Documentation: https://docs.flowcortex.example.com
- Status Page: https://status.flowcortex.example.com
