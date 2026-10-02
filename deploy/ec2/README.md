# FlowCortex EC2 deploy

Deploy FlowCortex L1 node and Explorer UI to the **same R&D EC2** used by eGenie / WiseEars, with the same SSH key and build routing.

## Quick start

```bash
cd /path/to/flow-cortex

# One-time: connection settings (same host as eGenie)
cp deploy/ec2/host.env.example deploy/ec2/host.env
# edit host.env if your key or host differ

chmod +x deploy/ec2/deploy-flowcortex.sh
./deploy/ec2/deploy-flowcortex.sh
```

## Connection (shared with eGenie)

| Setting | Default |
| --- | --- |
| Host | `100.52.147.238` |
| SSH key | `/Users/vijay/rnd/projects/VeerSetuHost.pem` |
| User | `ubuntu` |
| Explorer URL | `https://flowcortex.veerlabs.solutions` |
| L1 REST API | `https://flowcortex-api.veerlabs.solutions` |

Override via `deploy/ec2/host.env` or env vars:

```bash
DEPLOY_HOST=100.52.147.238 DEPLOY_KEY=/Users/vijay/rnd/projects/VeerSetuHost.pem ./deploy/ec2/deploy-flowcortex.sh
```

`host.env` is gitignored — copy from `host.env.example`.

## What gets installed

| Component | Location | Port |
| --- | --- | --- |
| **flowcortex-l1** | `/usr/local/bin/flowcortex-l1` | `127.0.0.1:8200` (REST) |
| **gRPC** | same process | `127.0.0.1:50052` |
| **flowcortex-explorer** | `/usr/local/bin/flowcortex-explorer` | `127.0.0.1:8201` |
| Source tree | `/opt/flow-cortex` | — |
| Node state | `/var/lib/flow-cortex/node_state.json` | — |
| Config | `/etc/flowcortex/l1.env`, `explorer.env` | — |
| systemd | `flowcortex-l1.service`, `flowcortex-explorer.service` | — |
| Caddy (if present) | Explorer + API hostnames | 443 |

No Docker infra is required — the L1 ledger runs in-process with disk-backed state.

## Linux builds (same as eGenie)

| `BUILD_MODE` | Behavior |
| --- | --- |
| `auto` (default) | Build on EC2 when SSH works; else LAN `192.168.29.78` |
| `ec2` | Force EC2 compile |
| `lan` | Build on `.78`, install when EC2 is back |

```bash
# EC2 stopped — build on LAN, then start EC2 and re-run:
BUILD_MODE=lan ./deploy/ec2/deploy-flowcortex.sh

# L1 only (no Explorer):
DEPLOY_EXPLORER=0 ./deploy/ec2/deploy-flowcortex.sh

# Skip compile (config/systemd only):
SKIP_BUILD=1 ./deploy/ec2/deploy-flowcortex.sh

# Force rebuild:
FORCE_BUILD=1 ./deploy/ec2/deploy-flowcortex.sh
```

## Port map on shared EC2

| Service | Port |
| --- | --- |
| FlowCortex L1 REST | `8200` |
| FlowCortex Explorer | `8201` |
| FlowCortex gRPC | `50052` (localhost) |

Does not conflict with WiseEars (`8080`), eGenie (`8081`), Srotiva (`8090`), EdgeFabric (`6060` / gRPC `50051`), KeyCortex (`8180`), or Velocity (`8190`).

## Prerequisites on EC2

Rust toolchain is installed automatically on first deploy. Caddy (from eGenie bootstrap) terminates TLS.

Fresh box bootstrap:

```bash
# From egenie repo
scp -i /Users/vijay/rnd/projects/VeerSetuHost.pem -r deploy/ec2 ubuntu@HOST:/tmp/egenie-ec2
ssh -i /Users/vijay/rnd/projects/VeerSetuHost.pem ubuntu@HOST 'sudo bash /tmp/egenie-ec2/bootstrap-host.sh'
```

## DNS

Add A records (same elastic IP as eGenie):

```
flowcortex.veerlabs.solutions      →  100.52.147.238
flowcortex-api.veerlabs.solutions  →  100.52.147.238
```

The Explorer proxies browser `/api/*` calls to the L1 node on localhost, so the Explorer hostname is enough for UI use. The API hostname exposes REST directly for integrations and `curl`.

## Verify

On the host:

```bash
curl http://127.0.0.1:8200/pool
curl http://127.0.0.1:8201/ | head
sudo journalctl -u flowcortex-l1 -f
sudo journalctl -u flowcortex-explorer -f
```

From your laptop (after DNS):

```bash
curl https://flowcortex-api.veerlabs.solutions/pool
open https://flowcortex.veerlabs.solutions
```

## Idle shutdown

If eGenie idle-shutdown is enabled, deploy adds ports **8200** and **8201** to `MONITOR_PORTS` in `/etc/egenie/idle-shutdown.conf`.

## Files in this directory

| File | Purpose |
| --- | --- |
| `host.env.example` | SSH host, key, public URLs, ports |
| `linux-build.sh` | EC2 vs LAN build routing |
| `deploy-flowcortex.sh` | Main deploy script |

## Related

- eGenie EC2 docs: `../egenie/deploy/ec2/README.md` (sibling repo)
- Local dev: `scripts/run_servers.sh`
