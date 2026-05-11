//! SSH client using ssh2

use std::io::{Read, Write};
use std::net::ToSocketAddrs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use ssh2::Session;
use tokio::sync::RwLock;

use agent_proto::TaskError;

pub struct SshClient {
    session: Arc<RwLock<Session>>,
    host: String,
    port: u16,
    user: String,
}

impl SshClient {
    pub async fn connect(
        host: &str,
        port: u16,
        user: &str,
        password: Option<&str>,
        key_path: Option<&str>,
    ) -> Result<Self, TaskError> {
        let addr = format!("{}:{}", host, port);
        let addrs: Vec<_> = addr.to_socket_addrs().map_err(|e| {
            TaskError::new("INVALID_ADDRESS", &e.to_string(), false)
        })?.collect();
        
        let socket = std::net::TcpStream::connect_timeout(
            &addrs[0],
            Duration::from_secs(10),
        )
        .map_err(|e| TaskError::new("CONNECT_FAILED", &e.to_string(), false))?;

        socket.set_read_timeout(Some(Duration::from_secs(30)))
            .map_err(|e| TaskError::new("TIMEOUT_SETUP", &e.to_string(), false))?;

        let mut session = Session::new().map_err(|e| {
            TaskError::new("SESSION_FAILED", &e.to_string(), false)
        })?;

        session.set_tcp_stream(socket);
        session
            .handshake()
            .map_err(|e| TaskError::new("HANDSHAKE_FAILED", &e.to_string(), false))?;

        // Authenticate
        if let Some(pass) = password {
            session
                .userauth_password(user, pass)
                .map_err(|e| TaskError::new("AUTH_FAILED", &e.to_string(), false))?;
        } else if let Some(key) = key_path {
            let key_file = Path::new(key);
            session
                .userauth_pubkey_file(user, None, key_file, None)
                .map_err(|e| TaskError::new("AUTH_KEY_FAILED", &e.to_string(), false))?;
        } else {
            return Err(TaskError::new(
                "NO_AUTH_METHOD",
                "Either password or key required",
                false,
            ));
        }

        if !session.authenticated() {
            return Err(TaskError::new(
                "AUTH_FAILED",
                "Authentication failed",
                false,
            ));
        }

        Ok(Self {
            session: Arc::new(RwLock::new(session)),
            host: host.to_string(),
            port,
            user: user.to_string(),
        })
    }

    pub async fn execute(&self, command: &str) -> Result<String, TaskError> {
        let session = self.session.read().await;

        let mut channel = session
            .channel_session()
            .map_err(|e| TaskError::new("CHANNEL_FAILED", &e.to_string(), true))?;

        channel
            .exec(command)
            .map_err(|e| TaskError::new("EXEC_FAILED", &e.to_string(), true))?;

        let mut output = String::new();
        channel
            .read_to_string(&mut output)
            .map_err(|e| TaskError::new("READ_FAILED", &e.to_string(), true))?;

        channel.wait_close().ok();

        Ok(output)
    }

    pub async fn execute_with_stdin(
        &self,
        command: &str,
        stdin: &str,
    ) -> Result<(String, String), TaskError> {
        let session = self.session.read().await;

        let mut channel = session
            .channel_session()
            .map_err(|e| TaskError::new("CHANNEL_FAILED", &e.to_string(), true))?;

        channel
            .exec(command)
            .map_err(|e| TaskError::new("EXEC_FAILED", &e.to_string(), true))?;

        channel
            .write_all(stdin.as_bytes())
            .map_err(|e| TaskError::new("WRITE_FAILED", &e.to_string(), true))?;
        channel.flush().ok();
        channel.send_eof().ok();

        let mut stdout = String::new();
        channel
            .read_to_string(&mut stdout)
            .map_err(|e| TaskError::new("READ_FAILED", &e.to_string(), true))?;

        let mut stderr = String::new();
        channel
            .stderr()
            .read_to_string(&mut stderr)
            .map_err(|e| TaskError::new("READ_STDERR", &e.to_string(), true))?;

        channel.wait_close().ok();

        Ok((stdout, stderr))
    }

    pub async fn upload_file(
        &self,
        local_path: &str,
        remote_path: &str,
    ) -> Result<(), TaskError> {
        use std::fs::File;
        use std::io::Read;

        let session = self.session.read().await;

        let local_file = File::open(local_path).map_err(|e| {
            TaskError::new("FILE_OPEN", &e.to_string(), false)
        })?;

        let sftp = session
            .sftp()
            .map_err(|e| TaskError::new("SFTP_INIT", &e.to_string(), true))?;

        let mut remote_file = sftp
            .create(Path::new(remote_path))
            .map_err(|e| TaskError::new("REMOTE_CREATE", &e.to_string(), true))?;

        let mut buffer = [0u8; 8192];
        let mut reader = local_file;

        loop {
            let n = reader.read(&mut buffer).map_err(|e| {
                TaskError::new("READ_LOCAL", &e.to_string(), true)
            })?;
            if n == 0 {
                break;
            }
            remote_file.write_all(&buffer[..n]).map_err(|e| {
                TaskError::new("WRITE_REMOTE", &e.to_string(), true)
            })?;
        }

        Ok(())
    }

    pub async fn download_file(
        &self,
        remote_path: &str,
        local_path: &str,
    ) -> Result<(), TaskError> {
        use std::fs::File;
        use std::io::Write;

        let session = self.session.read().await;

        let sftp = session
            .sftp()
            .map_err(|e| TaskError::new("SFTP_INIT", &e.to_string(), true))?;

        let mut remote_file = sftp
            .open(Path::new(remote_path))
            .map_err(|e| TaskError::new("REMOTE_OPEN", &e.to_string(), true))?;

        let mut local_file = File::create(local_path).map_err(|e| {
            TaskError::new("LOCAL_CREATE", &e.to_string(), false)
        })?;

        let mut buffer = [0u8; 8192];

        loop {
            let n = remote_file.read(&mut buffer).map_err(|e| {
                TaskError::new("READ_REMOTE", &e.to_string(), true)
            })?;
            if n == 0 {
                break;
            }
            local_file.write_all(&buffer[..n]).map_err(|e| {
                TaskError::new("WRITE_LOCAL", &e.to_string(), false)
            })?;
        }

        Ok(())
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn user(&self) -> &str {
        &self.user
    }
}

impl Clone for SshClient {
    fn clone(&self) -> Self {
        Self {
            session: self.session.clone(),
            host: self.host.clone(),
            port: self.port,
            user: self.user.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ssh_client_creation() {
        // This test will fail without SSH server, but ensures code compiles
        let result = SshClient::connect("localhost", 22, "test", Some("wrong"), None).await;
        assert!(result.is_err());
    }
}
