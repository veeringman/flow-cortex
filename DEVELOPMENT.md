# FlowCortex - Development Quick Start Guide

Complete guide for setting up, running, and developing FlowCortex locally or in Docker.

## Prerequisites

### Local Development
- **Rust** 1.56+ (install from https://rustup.rs)
- **Node.js** 16+ (for TypeScript examples)
- **Python** 3.7+ (for Python examples)
- **Docker** (optional, for containerized development)
- **Git**

### System Requirements
- **RAM**: Minimum 2GB (4GB recommended for compilation)
- **Disk**: 3GB free space for Rust target directory
- **CPU**: Multi-core recommended for faster compilation

## Quick Start (Local)

### 1. Clone and Setup

```bash
# Clone the repository
git clone https://github.com/your-org/flow-cortex.git
cd flow-cortex

# Initialize submodules (if any)
git submodule update --init --recursive
```

### 2. Build All Components

```bash
# Build everything in release mode
./scripts/build.sh

# Or build individual components
cd flowcortex-l1 && cargo build --release
cd ../explorer && cargo build --release
cd ../examples/l1-integration-clients/typescript && npm install && npm run build
```

### 3. Run Locally

```bash
# Start L1 node and explorer
./scripts/run_servers.sh

# Or run components separately
./scripts/start-l1-node.sh      # Terminal 1
./scripts/start-explorer.sh     # Terminal 2

# Run integration client examples
./scripts/run-client-examples.sh
```

## Docker Setup

### Quick Docker Start

```bash
# Build all Docker images
docker-compose build

# Start all services
docker-compose up

# Start services in background
docker-compose up -d

# View logs
docker-compose logs -f

# Stop and clean up
docker-compose down
```

### Individual Docker Services

```bash
# L1 Node only
docker run -p 3000:3000 -p 50051:50051 flowcortex-l1

# Explorer only
docker run -p 4000:4000 flowcortex-explorer

# Python client
docker run -v $(pwd):/workspace flowcortex-python python3 example.py

# Node.js client
docker run -it flowcortex-typescript npm run example:node
```

## Development Directory Structure

```
flow-cortex/
├── flowcortex-l1/              # L1 node (Rust)
├── flowcortex-l0/              # L0 proof system (Rust)
├── explorer/                   # Web UI (Rust/HTML/JS)
├── examples/
│   └── l1-integration-clients/ # Client examples
│       ├── rust-grpc/          # Rust gRPC client
│       ├── typescript/         # TypeScript/JS client
│       ├── python/             # Python client
│       └── curl/               # cURL examples
├── scripts/
│   ├── build.sh               # Build all components
│   ├── dev-setup.sh           # Development environment setup
│   ├── run_servers.sh         # Run L1 + Explorer
│   ├── start-l1-node.sh       # L1 node only
│   ├── start-explorer.sh      # Explorer only
│   ├── run-client-examples.sh # Run client examples
│   ├── docker-build.sh        # Build Docker images
│   └── ci/                    # CI/CD scripts
├── docker/
│   ├── Dockerfile.l1          # L1 node container
│   ├── Dockerfile.explorer    # Explorer container
│   ├── Dockerfile.typescript  # TypeScript client container
│   └── Dockerfile.python      # Python client container
├── docker-compose.yml         # Container orchestration
└── docs/
    ├── DEVELOPMENT.md         # Development guide
    ├── ARCHITECTURE.md        # System architecture
    ├── API.md                 # API reference
    └── DEPLOYMENT.md          # Deployment guide
```

## Common Development Tasks

### Build in Debug Mode (Faster Compilation)

```bash
cargo build          # Debug build
cargo run           # Run with debug info
```

### Run Tests

```bash
# L1 node tests
cd flowcortex-l1 && cargo test

# Explorer tests
cd explorer && cargo test

# Integration tests
cargo test --manifest-path flowcortex-l1/tests/e2e.rs

# All tests with logging
RUST_LOG=debug cargo test -- --nocapture
```

### Watch Mode (Auto-rebuild)

```bash
# Install cargo-watch
cargo install cargo-watch

# Watch and rebuild L1 node
cd flowcortex-l1
cargo watch -x build

# Watch and run L1 node
cargo watch -x "run --release"
```

### Code Formatting

```bash
# Format all Rust code
cargo fmt --all

# Check formatting without modifying
cargo fmt --all -- --check

# Check clippy lints
cargo clippy --all -- -D warnings
```

### API Documentation

```bash
# Generate and open documentation
cargo doc --open
```

## Environment Configuration

### L1 Node Environment Variables

```bash
# REST API bind address (default: 0.0.0.0:3000)
export BIND_ADDR=127.0.0.1:3000

# gRPC bind address (default: 0.0.0.0:50051)
export GRPC_ADDR=127.0.0.1:50051

# Block producer interval (default: 5000ms)
export BLOCK_INTERVAL=5000

# Run the node
cd flowcortex-l1 && cargo run --release
```

### Explorer Environment Variables

```bash
# Explorer UI bind address (default: 0.0.0.0:4000)
export BIND_ADDR=0.0.0.0:4000

# Run the explorer
cd explorer && cargo run --release
```

### Client Configuration

```bash
# Python client
export L1_URL=http://127.0.0.1:3000
export L1_GRPC_URL=127.0.0.1:50051

# TypeScript/Node.js
export L1_URL=http://127.0.0.1:3000
export L1_GRPC_URL=127.0.0.1:50051

# cURL
BASE_URL=http://127.0.0.1:3000 ./curl/run-examples.sh
```

## Troubleshooting

### Build Errors

**"cargo not found"**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

**"Out of disk space during build"**
```bash
# Clean build artifacts
cargo clean
# Try building again
```

**"LLVM error"**
```bash
# Update Rust
rustup update
# Clean and rebuild
cargo clean && cargo build --release
```

### Runtime Errors

**"Port already in use"**
```bash
# Find process using port 3000
lsof -i :3000
# Kill it
kill -9 PID

# Or use different port
export BIND_ADDR=0.0.0.0:3001
```

**"Connection refused"**
```bash
# Ensure L1 node is running
ps aux | grep flowcortex-l1
# Check if service is listening
netstat -tlnp | grep 3000
```

## Performance Tips

### Compilation Speed

```bash
# Use mold linker (faster linking on Linux)
RUSTFLAGS="-C link-arg=-fuse-ld=mold" cargo build --release

# Or use lld
RUSTFLAGS="-C link-arg=-fuse-ld=lld" cargo build --release

# Parallel compilation with more jobs
cargo build -j 4
```

### Runtime Performance

```bash
# Run with CPU optimization
CARGO_PROFILE_RELEASE_LTO=true cargo build --release

# Profile with flamegraph
cargo install flamegraph
cargo flamegraph --bin flowcortex-l1
```

## IDE Setup

### VSCode

1. Install Rust Analyzer extension
2. Install CodeLLDB for debugging
3. Copy `.vscode/settings.json` template:

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.checkOnSave.extraArgs": ["--all-targets", "--all-features"],
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

### IntelliJ IDEA / CLion

1. Install Rust plugin
2. Open project as Cargo workspace
3. Create run configuration:
   - Program: `flowcortex-l1`
   - Working directory: `$ProjectFileDir$`
   - Environment: `RUST_LOG=debug`

## Continuous Integration

### Local CI Checks

```bash
# Run all checks (format, lint, test)
./scripts/ci-check.sh

# Individual checks
cargo fmt --all -- --check  # Format check
cargo clippy --all          # Lint check
cargo test --all            # Test run
```

### GitHub Actions

See `.github/workflows/` for automated CI/CD configuration.

## Contributing

1. Create a feature branch: `git checkout -b feature/your-feature`
2. Make changes and commit: `git commit -am "Add feature"`
3. Push to branch: `git push origin feature/your-feature`
4. Create Pull Request with description

### Code Style

- Follow Rust conventions (enforced by `cargo fmt`)
- Address all clippy warnings
- Add tests for new functionality
- Update documentation

## Documentation

- **README.md** - Project overview
- **DEVELOPMENT.md** - This file
- **ARCHITECTURE.md** - System design
- **API.md** - API reference
- **examples/l1-integration-clients/INTEGRATION_GUIDE.md** - Client integration guide

## Support

- **Issues**: Report bugs on GitHub Issues
- **Discussions**: Ask questions in GitHub Discussions
- **Documentation**: Check docs/ directory
- **Examples**: See examples/l1-integration-clients/

## License

See LICENSE file in root directory.

## Quick Reference Commands

```bash
# Full setup from scratch
git clone <repo> && cd flow-cortex
./scripts/dev-setup.sh
./scripts/build.sh
./scripts/run_servers.sh

# Docker quick start
docker-compose up --build

# Run all tests
cargo test --all

# Format and lint
cargo fmt --all && cargo clippy --all

# Integration examples
cd examples/l1-integration-clients
./curl/run-examples.sh         # cURL
python3 python/example.py      # Python
npm run example:node           # TypeScript
cargo run --release            # Rust (from rust-grpc dir)
```
