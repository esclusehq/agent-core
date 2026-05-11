//! Runtime detection (Docker/Podman)

use std::path::Path;
use std::process::Command;

use bollard::Docker;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeType {
    Docker,
    Podman,
    None,
}

pub struct RuntimeDetector {
    pub runtime: RuntimeType,
    pub version: Option<String>,
    pub available: bool,
    docker_client: Option<Docker>,
}

impl RuntimeDetector {
    pub fn detect() -> Self {
        if let Some(docker) = Self::detect_docker() {
            return docker;
        }

        if let Some(podman) = Self::detect_podman() {
            return podman;
        }

        Self {
            runtime: RuntimeType::None,
            version: None,
            available: false,
            docker_client: None,
        }
    }

    fn detect_docker() -> Option<Self> {
        let docker_path = which::which("docker").ok()?;

        let output = Command::new(&docker_path)
            .args(["version", "--format", "{{.Server.Version}}"])
            .output()
            .ok()?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();

            let socket_path = Path::new("/var/run/docker.sock");
            let socket_available = socket_path.exists();

            let docker_client = if socket_available {
                Docker::connect_with_local_defaults().ok()
            } else {
                None
            };

            return Some(Self {
                runtime: RuntimeType::Docker,
                version: Some(version),
                available: socket_available,
                docker_client,
            });
        }

        None
    }

    fn detect_podman() -> Option<Self> {
        let podman_path = which::which("podman").ok()?;

        let output = Command::new(&podman_path)
            .args(["version", "--format", "{{.Version}}"])
            .output()
            .ok()?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();

            let socket_path = Path::new("/run/user/0/podman/podman.sock");
            let socket_available = socket_path.exists();

            return Some(Self {
                runtime: RuntimeType::Podman,
                version: Some(version),
                available: socket_available,
                docker_client: None,
            });
        }

        None
    }

    pub fn docker(&self) -> Option<&Docker> {
        self.docker_client.as_ref()
    }

    pub fn is_docker(&self) -> bool {
        self.runtime == RuntimeType::Docker
    }

    pub fn is_podman(&self) -> bool {
        self.runtime == RuntimeType::Podman
    }

    pub fn runtime_name(&self) -> &str {
        match self.runtime {
            RuntimeType::Docker => "docker",
            RuntimeType::Podman => "podman",
            RuntimeType::None => "none",
        }
    }
}

impl Default for RuntimeDetector {
    fn default() -> Self {
        Self::detect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_detection() {
        let detector = RuntimeDetector::detect();

        // On Linux CI, should detect either Docker or Podman or None
        assert!(matches!(
            detector.runtime,
            RuntimeType::Docker | RuntimeType::Podman | RuntimeType::None
        ));
    }
}
