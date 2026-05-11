# Agent Core — Escluse Agent Framework

Agent Core adalah Cargo workspace dengan 12 crate yang menyediakan fitur-fitur untuk membangun agent (Web, Desktop, Mobile).

## Struktur Workspace

```
agent-core/
├── Cargo.toml              # Workspace root
├── README.md               # Dokumentasi utama
├── IMPLEMENTATION_PLAN.md  # Rencana implementasi
├── crates/
│   ├── agent-proto         # Protocol types & messages
│   ├── agent-config        # Configuration management
│   ├── agent-security      # JWT, rate limiting, audit
│   ├── agent-event         # Internal pub/sub
│   ├── agent-health        # Circuit breaker, retry
│   ├── agent-capability    # Capability registry
│   ├── agent-task          # Task queue & dispatcher
│   ├── agent-metrics       # Metrics collection
│   ├── agent-runtime       # Docker/Podman detection
│   ├── agent-ssh           # SSH client & pool
│   ├── agent-backup        # Compression utilities
│   └── agent-rcon          # RCON protocol
```

## Fitur per Crate

### Phase 1: Foundation
| Crate | Fitur |
|-------|-------|
| **agent-proto** | Task, TaskResult, TaskError, TaskPriority, WebSocket messages, protocol version |
| **agent-config** | Load config dari env/files, validasi, SecretString |
| **agent-health** | Circuit breaker, retry with backoff, health monitoring |

### Phase 2: Cross-cutting
| Crate | Fitur |
|-------|-------|
| **agent-security** | JWT validation, rate limiting (token bucket), audit logging |
| **agent-event** | Pub/sub event bus, task lifecycle events |
| **agent-capability** | Capability registry, task-capability matcher |

### Phase 3: Core Engine
| Crate | Fitur |
|-------|-------|
| **agent-task** | Priority queue, task dispatcher, concurrency control |
| **agent-metrics** | System metrics (CPU, memory, disk, network) |

### Phase 4: Runtime Adapters
| Crate | Fitur |
|-------|-------|
| **agent-runtime** | Docker/Podman detection |
| **agent-ssh** | SSH client, SFTP, connection pooling |

### Phase 5: Operations
| Crate | Fitur |
|-------|-------|
| **agent-backup** | Compression (zstd, gzip) |
| **agent-rcon** | RCON protocol client |

## Build

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

## Optimasi Release

Profile release sudah di-optimize untuk production:
- `opt-level = 3` (maximum)
- `lto = "fat"` (full LTO)
- `panic = "abort"` (smaller binary)

## Crate Usage Matrix

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

## Contoh Penggunaan

```rust
use agent_proto::{Task, TaskPriority};
use agent_config::{load, validate};
use agent_task::{TaskQueue, dispatch_task};
use agent_health::{CircuitBreaker, RetryConfig};
use agent_security::{JwtValidator, RateLimiter};
```

## Lisensi

MIT
