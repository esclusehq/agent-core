//! Agent Capability - Capability registry and matcher

use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Capability {
    Docker,
    Podman,
    LocalCommand,
    SSH,
    SFTP,
    ServerCreate,
    ServerStart,
    ServerStop,
    ServerRestart,
    ServerDelete,
    ServerLogs,
    ServerCommand,
    BackupCreate,
    BackupRestore,
    Metrics,
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Capability::Docker => write!(f, "docker"),
            Capability::Podman => write!(f, "podman"),
            Capability::LocalCommand => write!(f, "local_command"),
            Capability::SSH => write!(f, "ssh"),
            Capability::SFTP => write!(f, "sftp"),
            Capability::ServerCreate => write!(f, "server_create"),
            Capability::ServerStart => write!(f, "server_start"),
            Capability::ServerStop => write!(f, "server_stop"),
            Capability::ServerRestart => write!(f, "server_restart"),
            Capability::ServerDelete => write!(f, "server_delete"),
            Capability::ServerLogs => write!(f, "server_logs"),
            Capability::ServerCommand => write!(f, "server_command"),
            Capability::BackupCreate => write!(f, "backup_create"),
            Capability::BackupRestore => write!(f, "backup_restore"),
            Capability::Metrics => write!(f, "metrics"),
        }
    }
}

pub struct CapabilityRegistry {
    capabilities: HashSet<Capability>,
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self {
            capabilities: HashSet::new(),
        }
    }

    pub fn register(&mut self, cap: Capability) {
        self.capabilities.insert(cap);
    }

    pub fn remove(&mut self, cap: &Capability) {
        self.capabilities.remove(cap);
    }

    pub fn has(&self, cap: &Capability) -> bool {
        self.capabilities.contains(cap)
    }

    pub fn has_all(&self, required: &[Capability]) -> bool {
        required.iter().all(|c| self.has(c))
    }

    pub fn to_string_list(&self) -> Vec<String> {
        self.capabilities.iter().map(|c| c.to_string()).collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Capability> {
        self.capabilities.iter()
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn required_for(task_type: &str) -> Vec<Capability> {
    match task_type {
        "server.create" => vec![Capability::ServerCreate],
        "server.start" => vec![Capability::ServerStart],
        "server.stop" => vec![Capability::ServerStop],
        "server.restart" => vec![Capability::ServerRestart],
        "server.delete" => vec![Capability::ServerDelete],
        "server.logs" => vec![Capability::ServerLogs],
        "server.command" => vec![Capability::ServerCommand],
        "backup.create" => vec![Capability::BackupCreate],
        "backup.restore" => vec![Capability::BackupRestore],
        "ssh.execute" => vec![Capability::SSH],
        "sftp.upload" => vec![Capability::SFTP],
        "sftp.download" => vec![Capability::SFTP],
        "metrics.report" => vec![Capability::Metrics],
        _ => vec![],
    }
}

pub fn can_handle(registry: &CapabilityRegistry, task_type: &str) -> bool {
    let required = required_for(task_type);
    !required.is_empty() && registry.has_all(&required)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_registry() {
        let mut registry = CapabilityRegistry::new();

        registry.register(Capability::Docker);
        assert!(registry.has(&Capability::Docker));
        assert!(!registry.has(&Capability::Podman));
    }

    #[test]
    fn test_required_for() {
        assert!(required_for("server.start").contains(&Capability::ServerStart));
        assert!(required_for("ssh.execute").contains(&Capability::SSH));
    }

    #[test]
    fn test_can_handle() {
        let mut registry = CapabilityRegistry::new();
        registry.register(Capability::ServerStart);

        assert!(can_handle(&registry, "server.start"));
        assert!(!can_handle(&registry, "server.stop"));
    }
}
