# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [v0.0.1] - 2026-05-12

### Added

**Workspace** — Cargo workspace with 12 crates under `crates/` directory.

| Crate | Version | Description |
|-------|---------|-------------|
| `agent-proto` | 0.0.1 | Task, TaskResult, TaskError, TaskPriority, WebSocket message types, protocol version |
| `agent-config` | 0.0.1 | Config loading from env/files, validation, SecretString |
| `agent-health` | 0.0.1 | Circuit breaker, retry with backoff, health monitoring |
| `agent-security` | 0.0.1 | JWT validation, rate limiting (token bucket), audit logging |
| `agent-event` | 0.0.1 | Pub/sub event bus, task lifecycle events |
| `agent-capability` | 0.0.1 | Capability registry, task-capability matcher |
| `agent-task` | 0.0.1 | Priority queue, task dispatcher, concurrency control |
| `agent-metrics` | 0.0.1 | System metrics (CPU, memory, disk, network) |
| `agent-runtime` | 0.0.1 | Docker/Podman detection |
| `agent-ssh` | 0.0.1 | SSH client, SFTP, connection pooling |
| `agent-backup` | 0.0.1 | Compression (zstd, gzip) |
| `agent-rcon` | 0.0.1 | RCON protocol client |

### Shared Workspace Dependencies

| Dependency | Version | Usage |
|------------|---------|-------|
| `tokio` | 1 (full) | Async runtime for all crates |
| `serde` | 1 | Serialization/deserialization |
| `serde_json` | 1 | JSON parsing |
| `uuid` | 1 | Task IDs, session IDs |
| `chrono` | 0.4 | Timestamp handling |
| `thiserror` | 2 | Error types |
| `tracing` | 0.1 | Logging |
| `async-trait` | 0.1 | Async trait support |

### Crate Dependency Graph

```
agent-proto (no dependencies)
├── agent-config → agent-proto
├── agent-health → agent-proto, agent-config
├── agent-event → agent-proto
├── agent-security → agent-proto, agent-config
├── agent-capability → agent-proto, agent-event
├── agent-task → agent-proto, agent-event, agent-capability
├── agent-metrics → agent-proto
├── agent-runtime → agent-proto, agent-config
├── agent-ssh → agent-proto, agent-config
├── agent-backup → agent-proto
└── agent-rcon → agent-proto
```

### Build

```bash
# Development (fast)
cargo build --workspace

# Production (optimized)
cargo build --release --workspace

# Run tests
cargo test --workspace

# Test production
cargo test --release --workspace
```

### Release Profile

The `profile.release` is optimized for production:

```toml
opt-level = 3      # Maximum optimization
lto = "fat"        # Full link-time optimization
codegen-units = 1  # Maximum optimization at cost of compile time
strip = true       # Strip debug symbols
panic = "abort"    # Smaller binary, no stack unwinding
overflow-checks = false
```

### Crate Usage Matrix

| Crate | Web Agent | Desktop Agent |
|-------|:---------:|:-------------:|
| agent-proto | ✅ | ✅ |
| agent-config | ✅ | ✅ |
| agent-security | ✅ | ✅ |
| agent-event | ✅ | ✅ |
| agent-health | ✅ | ✅ |
| agent-capability | ✅ | ✅ |
| agent-task | ✅ | ✅ |
| agent-metrics | ✅ | ✅ |
| agent-runtime | ✅ | ❌ |
| agent-ssh | ❌ | ✅ |
| agent-backup | ✅ | ❌ |
| agent-rcon | ✅ | ✅ |

### License

MIT