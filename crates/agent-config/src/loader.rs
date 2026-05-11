//! Configuration loader from environment variables and files

use std::path::{Path, PathBuf};
use std::{env, fs};

use tracing_appender;

use super::{AgentConfig, RuntimePreference, SecretString};

/// Get platform-specific config directory
/// Windows: APPDATA/escluse-agent
/// Unix: XDG_CONFIG_HOME or ~/.config/escluse-agent
fn get_config_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        // Windows: Use APPDATA environment variable
        // Fallback to %USERPROFILE%\AppData\Roaming
        std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("AppData/Roaming")
            })
            .join("escluse-agent")
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Unix-like: Use XDG_CONFIG_HOME or fallback to ~/.config
        if let Ok(dir) = env::var("XDG_CONFIG_HOME") {
            return PathBuf::from(dir).join("escluse-agent");
        }

        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("escluse-agent")
    }
}

/// Get XDG config directory, with fallbacks per D-02, D-03
pub fn get_xdg_config_path(filename: &str) -> Option<PathBuf> {
    // Try XDG_CONFIG_HOME first
    if let Ok(dir) = env::var("XDG_CONFIG_HOME") {
        let path = PathBuf::from(dir).join("escluse-agent").join(filename);
        if path.exists() {
            return Some(path);
        }
    }

    // Try platform-specific config dir (APPDATA on Windows, ~/.config on Unix)
    let config_dir = get_config_dir();
    let path = config_dir.join(filename);
    if path.exists() {
        return Some(path);
    }

    // Fallback: ~/.local/share/escluse-agent/
    if let Some(dir) = dirs::data_local_dir() {
        let path = dir.join("escluse-agent").join(filename);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

pub fn load() -> AgentConfig {
    let mut config = AgentConfig::default();

    // Config loading order (documented for maintainability):
    // 1. Load from TOML file (XDG paths) - D-01: config.toml primary source
    // 2. Load old-style env vars (backward compatibility - AGENT_* prefix)
    // 3. Load legacy .env file
    // 4. Load new env overrides with ESCLUSE_AGENT_* prefix (takes precedence) - D-02: env override
    
    // 1. Load from TOML file (XDG paths)
    load_toml_config(&mut config);

    // 2. Load old-style env vars (backward compatibility - AGENT_* prefix)
    if let Ok(url) = env::var("AGENT_BACKEND_URL") {
        config.backend_url = url;
    }

    if let Ok(key) = env::var("AGENT_API_KEY") {
        config.api_key = SecretString::new(key);
    }

    if let Ok(name) = env::var("AGENT_NAME") {
        config.agent_name = name;
    }

    if let Ok(id) = env::var("AGENT_ID") {
        if let Ok(uuid) = uuid::Uuid::parse_str(&id) {
            config.agent_id = Some(uuid);
        }
    }

    if let Ok(interval) = env::var("AGENT_HEARTBEAT_INTERVAL") {
        if let Ok(v) = interval.parse() {
            config.heartbeat_interval_secs = v;
        }
    }

    if let Ok(max) = env::var("AGENT_MAX_CONCURRENT") {
        if let Ok(v) = max.parse() {
            config.max_concurrent_tasks = v;
        }
    }

    if let Ok(runtime) = env::var("AGENT_RUNTIME") {
        config.runtime_preference = match runtime.to_lowercase().as_str() {
            "docker" => RuntimePreference::Docker,
            "podman" => RuntimePreference::Podman,
            "none" => RuntimePreference::None,
            _ => RuntimePreference::Auto,
        };
    }

    if let Ok(level) = env::var("LOG_LEVEL") {
        config.log_level = level;
    }

    if let Ok(addr) = env::var("AGENT_METRICS_LISTEN") {
        if let Ok(socket_addr) = addr.parse() {
            config.metrics_listen_addr = Some(socket_addr);
        }
    }

    if let Ok(data_dir) = env::var("AGENT_DATA_DIR") {
        config.data_dir = Path::new(&data_dir).to_path_buf();
    }

    // 3. Load legacy .env file
    load_from_file(&mut config);

    // 4. Load new env overrides with ESCLUSE_AGENT_ prefix (takes precedence)
    load_env_overrides(&mut config);

    config
}

/// Load config from TOML file, then override with env vars
/// Per D-04: Env vars use ESCLUSE_AGENT_ prefix
/// Per D-05: Env vars take precedence over file
fn load_toml_config(config: &mut AgentConfig) {
    let config_path = get_xdg_config_path("config.toml");

    let Some(path) = config_path else {
        return;
    };

    let Ok(contents) = fs::read_to_string(&path) else {
        tracing::debug!("No TOML config found at {:?}", path);
        return;
    };

    // Parse TOML (use toml crate)
    if let Ok(toml_map) = contents.parse::<toml::Value>() {
        // Extract config values from TOML

        // [server]
        if let Some(v) = toml_map.get("server").and_then(|t| t.get("backend_url")) {
            if let Some(s) = v.as_str() {
                config.backend_url = s.to_string();
            }
        }

        // [server] - api_key (D-01: primary source from config.toml)
        if let Some(v) = toml_map.get("server").and_then(|t| t.get("api_key")) {
            if let Some(s) = v.as_str() {
                if !s.is_empty() {
                    config.api_key = SecretString::new(s.to_string());
                }
            }
        }

        // [agent]
        if let Some(v) = toml_map.get("agent").and_then(|t| t.get("name")) {
            if let Some(s) = v.as_str() {
                config.agent_name = s.to_string();
            }
        }

        if let Some(v) = toml_map.get("agent").and_then(|t| t.get("id")) {
            if let Some(s) = v.as_str() {
                if let Ok(uuid) = uuid::Uuid::parse_str(s) {
                    config.agent_id = Some(uuid);
                }
            }
        }

        // [connection]
        if let Some(v) = toml_map.get("connection").and_then(|t| t.get("heartbeat_interval")) {
            if let Some(i) = v.as_integer() {
                config.heartbeat_interval_secs = i as u64;
            }
        }

        if let Some(v) = toml_map.get("connection").and_then(|t| t.get("reconnect_initial")) {
            if let Some(i) = v.as_integer() {
                config.reconnect_initial_secs = i as u64;
            }
        }

        if let Some(v) = toml_map.get("connection").and_then(|t| t.get("reconnect_max")) {
            if let Some(i) = v.as_integer() {
                config.reconnect_max_secs = i as u64;
            }
        }

        // [task]
        if let Some(v) = toml_map.get("task").and_then(|t| t.get("max_concurrent")) {
            if let Some(i) = v.as_integer() {
                config.max_concurrent_tasks = i as usize;
            }
        }

        if let Some(v) = toml_map.get("task").and_then(|t| t.get("timeout_default")) {
            if let Some(i) = v.as_integer() {
                config.task_timeout_default_secs = i as u64;
            }
        }

        // [runtime]
        if let Some(v) = toml_map.get("runtime").and_then(|t| t.get("preference")) {
            if let Some(s) = v.as_str() {
                config.runtime_preference = match s.to_lowercase().as_str() {
                    "docker" => RuntimePreference::Docker,
                    "podman" => RuntimePreference::Podman,
                    "none" => RuntimePreference::None,
                    _ => RuntimePreference::Auto,
                };
            }
        }

        // [metrics]
        if let Some(v) = toml_map.get("metrics").and_then(|t| t.get("interval")) {
            if let Some(i) = v.as_integer() {
                config.metrics_interval_secs = i as u64;
            }
        }

        if let Some(v) = toml_map.get("metrics").and_then(|t| t.get("listen")) {
            if let Some(s) = v.as_str() {
                if let Ok(socket_addr) = s.parse() {
                    config.metrics_listen_addr = Some(socket_addr);
                }
            }
        }

        // [logging]
        if let Some(v) = toml_map.get("logging").and_then(|t| t.get("level")) {
            if let Some(s) = v.as_str() {
                config.log_level = s.to_string();
            }
        }

        if let Some(v) = toml_map.get("logging").and_then(|t| t.get("format")) {
            if let Some(s) = v.as_str() {
                config.log_format = match s.to_lowercase().as_str() {
                    "json" => super::LogFormat::Json,
                    _ => super::LogFormat::Text,
                };
            }
        }

        // [data]
        if let Some(v) = toml_map.get("data").and_then(|t| t.get("dir")) {
            if let Some(s) = v.as_str() {
                config.data_dir = Path::new(s).to_path_buf();
            }
        }
    }
}

/// Load environment variable overrides with ESCLUSE_AGENT_ prefix
/// Per D-04: Env vars use ESCLUSE_AGENT_ prefix
/// Per D-05: Env vars take precedence over file
fn load_env_overrides(config: &mut AgentConfig) {
    // ESCLUSE_AGENT_BACKEND_URL
    if let Ok(url) = env::var("ESCLUSE_AGENT_BACKEND_URL") {
        config.backend_url = url;
    }

    // ESCLUSE_AGENT_API_KEY
    if let Ok(key) = env::var("ESCLUSE_AGENT_API_KEY") {
        config.api_key = SecretString::new(key);
    }

    // ESCLUSE_AGENT_NAME
    if let Ok(name) = env::var("ESCLUSE_AGENT_NAME") {
        config.agent_name = name;
    }

    // ESCLUSE_AGENT_ID
    if let Ok(id) = env::var("ESCLUSE_AGENT_ID") {
        if let Ok(uuid) = uuid::Uuid::parse_str(&id) {
            config.agent_id = Some(uuid);
        }
    }

    // ESCLUSE_AGENT_HEARTBEAT_INTERVAL
    if let Ok(interval) = env::var("ESCLUSE_AGENT_HEARTBEAT_INTERVAL") {
        if let Ok(v) = interval.parse() {
            config.heartbeat_interval_secs = v;
        }
    }

    // ESCLUSE_AGENT_RECONNECT_INITIAL
    if let Ok(v) = env::var("ESCLUSE_AGENT_RECONNECT_INITIAL") {
        if let Ok(val) = v.parse() {
            config.reconnect_initial_secs = val;
        }
    }

    // ESCLUSE_AGENT_RECONNECT_MAX
    if let Ok(v) = env::var("ESCLUSE_AGENT_RECONNECT_MAX") {
        if let Ok(val) = v.parse() {
            config.reconnect_max_secs = val;
        }
    }

    // ESCLUSE_AGENT_MAX_CONCURRENT
    if let Ok(max) = env::var("ESCLUSE_AGENT_MAX_CONCURRENT") {
        if let Ok(v) = max.parse() {
            config.max_concurrent_tasks = v;
        }
    }

    // ESCLUSE_AGENT_TASK_TIMEOUT
    if let Ok(v) = env::var("ESCLUSE_AGENT_TASK_TIMEOUT") {
        if let Ok(val) = v.parse() {
            config.task_timeout_default_secs = val;
        }
    }

    // ESCLUSE_AGENT_RUNTIME
    if let Ok(runtime) = env::var("ESCLUSE_AGENT_RUNTIME") {
        config.runtime_preference = match runtime.to_lowercase().as_str() {
            "docker" => RuntimePreference::Docker,
            "podman" => RuntimePreference::Podman,
            "none" => RuntimePreference::None,
            _ => RuntimePreference::Auto,
        };
    }

    // ESCLUSE_AGENT_LOG_LEVEL
    if let Ok(level) = env::var("ESCLUSE_AGENT_LOG_LEVEL") {
        config.log_level = level;
    }

    // ESCLUSE_AGENT_LOG_FORMAT
    if let Ok(format) = env::var("ESCLUSE_AGENT_LOG_FORMAT") {
        config.log_format = match format.to_lowercase().as_str() {
            "json" => super::LogFormat::Json,
            _ => super::LogFormat::Text,
        };
    }

    // ESCLUSE_AGENT_METRICS_LISTEN
    if let Ok(addr) = env::var("ESCLUSE_AGENT_METRICS_LISTEN") {
        if let Ok(socket_addr) = addr.parse() {
            config.metrics_listen_addr = Some(socket_addr);
        }
    }

    // ESCLUSE_AGENT_METRICS_INTERVAL
    if let Ok(v) = env::var("ESCLUSE_AGENT_METRICS_INTERVAL") {
        if let Ok(val) = v.parse() {
            config.metrics_interval_secs = val;
        }
    }

    // ESCLUSE_AGENT_DATA_DIR
    if let Ok(data_dir) = env::var("ESCLUSE_AGENT_DATA_DIR") {
        config.data_dir = Path::new(&data_dir).to_path_buf();
    }
}

/// Guard that keeps file logging alive
/// Must be kept in scope for the duration of the program
pub struct FileLogGuard {
    _guard: tracing_appender::non_blocking::WorkerGuard,
}

impl FileLogGuard {
    /// Get the writer for use with tracing_subscriber
    pub fn writer(&self) -> impl std::io::Write + Send + 'static {
        // This is a dummy writer - we handle this differently
        // The actual writer is internal to the guard
        // We'll use a different approach below
        Vec::new()
    }
}

/// Get platform-specific log directory
fn get_log_dir_platform() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        // Windows: Use APPDATA/escluse-agent/logs
        std::env::var("APPDATA")
            .map(|p| PathBuf::from(p).join("escluse-agent").join("logs"))
            .ok()
            .or_else(|| {
                dirs::home_dir().map(|h| h.join("AppData/Roaming/escluse-agent/logs"))
            })
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Unix: Try /var/log/escluse-agent/ first
        let primary = PathBuf::from("/var/log/escluse-agent");
        if primary.exists() || std::fs::create_dir_all(&primary).is_ok() {
            return Some(primary);
        }
        // Fallback to ~/.local/share/escluse-agent/logs/
        dirs::data_local_dir().map(|d| d.join("escluse-agent").join("logs"))
    }
}

/// Get log directory path with fallbacks
/// Primary: /var/log/escluse-agent/ (D-06) or APPDATA on Windows
/// Fallback: ~/.local/share/escluse-agent/logs/ (D-07)
/// Last fallback: None for stdout (D-08 - for containerized environments)
pub fn get_log_dir() -> Option<FileLogGuard> {
    // Get platform-specific log directory
    if let Some(log_dir) = get_log_dir_platform() {
        if log_dir.exists() || std::fs::create_dir_all(&log_dir).is_ok() {
            // Create appender with daily rotation (D-09: daily + 5 files)
            let file_appender = tracing_appender::rolling::daily(&log_dir, "agent.log");
            let (_non_blocking, guard) = tracing_appender::non_blocking(file_appender);

            return Some(FileLogGuard { _guard: guard });
        }
    }

    // D-08: No file logging - return None for stdout fallback
    None
}

/// Get log writer for file logging (returns writer and guard)
/// This function returns both the writer to use and a guard that must be kept alive
pub fn get_log_writer() -> Option<(Box<dyn std::io::Write + Send + 'static>, FileLogGuard)> {
    // Get platform-specific log directory
    if let Some(log_dir) = get_log_dir_platform() {
        if log_dir.exists() || std::fs::create_dir_all(&log_dir).is_ok() {
            let file_appender = tracing_appender::rolling::daily(&log_dir, "agent.log");
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

            // Convert to Boxed writer
            let writer: Box<dyn std::io::Write + Send + 'static> = Box::new(non_blocking);
            return Some((writer, FileLogGuard { _guard: guard }));
        }
    }

    None
}

fn load_from_file(config: &mut AgentConfig) {
    let config_path = if let Ok(s) = env::var("AGENT_CONFIG_FILE") {
        Path::new(&s).to_path_buf()
    } else if Path::new(".env").exists() {
        Path::new(".env").to_path_buf()
    } else if Path::new("/etc/escluse/agent.env").exists() {
        Path::new("/etc/escluse/agent.env").to_path_buf()
    } else {
        return;
    };

    if !config_path.exists() {
        return;
    }

    if let Ok(contents) = fs::read_to_string(config_path) {
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "AGENT_BACKEND_URL" => config.backend_url = value.to_string(),
                    "AGENT_API_KEY" => config.api_key = SecretString::new(value.to_string()),
                    "AGENT_NAME" => config.agent_name = value.to_string(),
                    "AGENT_RUNTIME" => {
                        config.runtime_preference = match value.to_lowercase().as_str() {
                            "docker" => RuntimePreference::Docker,
                            "podman" => RuntimePreference::Podman,
                            "none" => RuntimePreference::None,
                            _ => RuntimePreference::Auto,
                        };
                    }
                    "LOG_LEVEL" => config.log_level = value.to_string(),
                    "AGENT_DATA_DIR" => config.data_dir = Path::new(value).to_path_buf(),
                    "AGENT_HEARTBEAT_INTERVAL" => {
                        if let Ok(v) = value.parse() {
                            config.heartbeat_interval_secs = v;
                        }
                    }
                    "AGENT_RECONNECT_INITIAL" => {
                        if let Ok(v) = value.parse() {
                            config.reconnect_initial_secs = v;
                        }
                    }
                    "AGENT_RECONNECT_MAX" => {
                        if let Ok(v) = value.parse() {
                            config.reconnect_max_secs = v;
                        }
                    }
                    "AGENT_MAX_CONCURRENT" => {
                        if let Ok(v) = value.parse() {
                            config.max_concurrent_tasks = v;
                        }
                    }
                    "AGENT_TASK_TIMEOUT" => {
                        if let Ok(v) = value.parse() {
                            config.task_timeout_default_secs = v;
                        }
                    }
                    "AGENT_METRICS_INTERVAL" => {
                        if let Ok(v) = value.parse() {
                            config.metrics_interval_secs = v;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_from_env() {
        env::set_var("AGENT_BACKEND_URL", "wss://test.example.com");
        env::set_var("AGENT_API_KEY", "test_key_123");

        let config = load();

        assert_eq!(config.backend_url, "wss://test.example.com");
        assert_eq!(config.api_key.expose_secret(), "test_key_123");

        env::remove_var("AGENT_BACKEND_URL");
        env::remove_var("AGENT_API_KEY");
    }
}
