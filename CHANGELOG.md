# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-05-12

### Added

**Phase 1: Foundation**
- `agent-proto` - Task, TaskResult, TaskError, TaskPriority, WebSocket messages, protocol version
- `agent-config` - Configuration management, env/files loading, validation, SecretString
- `agent-health` - Circuit breaker, retry with backoff, health monitoring

**Phase 2: Cross-cutting**
- `agent-security` - JWT validation, rate limiting (token bucket), audit logging
- `agent-event` - Pub/sub event bus, task lifecycle events
- `agent-capability` - Capability registry, task-capability matcher

**Phase 3: Core Engine**
- `agent-task` - Priority queue, task dispatcher, concurrency control
- `agent-metrics` - System metrics (CPU, memory, disk, network)

**Phase 4: Runtime Adapters**
- `agent-runtime` - Docker/Podman detection
- `agent-ssh` - SSH client, SFTP, connection pooling

**Phase 5: Operations**
- `agent-backup` - Compression (zstd, gzip)
- `agent-rcon` - RCON protocol client

### Technical Details

#### Crate Dependencies
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

#### Workspace Optimization
- Release profile with `opt-level = 3`
- Full LTO (`lto = "fat"`)
- Panic handler set to `abort` for smaller binaries