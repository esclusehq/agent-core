//! RCON client implementation

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use agent_proto::TaskError;

const RCON_TYPE_COMMAND: i32 = 2;
const RCON_AUTH: i32 = 3;

pub struct RconClient {
    stream: TcpStream,
    packet_id: i32,
}

impl RconClient {
    pub fn connect(host: &str, port: u16, password: &str) -> Result<Self, TaskError> {
        let addr = format!("{}:{}", host, port);

        let addrs: Vec<_> = addr
            .to_socket_addrs()
            .map_err(|e| TaskError::new("INVALID_ADDRESS", &e.to_string(), false))?
            .collect();

        let stream = TcpStream::connect_timeout(&addrs[0], Duration::from_secs(5))
            .map_err(|e| TaskError::new("CONNECT_FAILED", &e.to_string(), false))?;

        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .map_err(|e| TaskError::new("TIMEOUT_SETUP", &e.to_string(), false))?;

        let mut client = Self {
            stream,
            packet_id: 1,
        };

        // Authenticate
        client.authenticate(password)?;

        Ok(client)
    }

    fn authenticate(&mut self, password: &str) -> Result<(), TaskError> {
        let request_id = self.next_id();

        self.send_packet(RCON_AUTH, request_id, password)
            .map_err(|e| TaskError::new("SEND_AUTH_FAILED", &e.to_string(), false))?;

        let response = self
            .read_packet()
            .map_err(|e| TaskError::new("READ_AUTH_RESPONSE", &e.to_string(), true))?;

        if response.id != request_id {
            return Err(TaskError::new(
                "AUTH_FAILED",
                "Authentication failed - invalid packet ID",
                false,
            ));
        }

        Ok(())
    }

    pub fn execute(&mut self, command: &str) -> Result<String, TaskError> {
        let request_id = self.next_id();

        self.send_packet(RCON_TYPE_COMMAND, request_id, command)
            .map_err(|e| TaskError::new("SEND_FAILED", &e.to_string(), false))?;

        let response = self
            .read_packet()
            .map_err(|e| TaskError::new("READ_FAILED", &e.to_string(), true))?;

        if response.id != request_id {
            return Err(TaskError::new(
                "INVALID_RESPONSE",
                "Response packet ID mismatch",
                true,
            ));
        }

        Ok(response.payload)
    }

    fn next_id(&mut self) -> i32 {
        let id = self.packet_id;
        self.packet_id = self.packet_id.wrapping_add(1);
        id
    }

    fn send_packet(&mut self, packet_type: i32, id: i32, payload: &str) -> std::io::Result<()> {
        let payload_bytes = payload.as_bytes();
        let length = 4 + 4 + 4 + payload_bytes.len() + 2;

        let mut packet = Vec::with_capacity(length + 4);

        packet.extend_from_slice(&(length as i32).to_le_bytes());
        packet.extend_from_slice(&id.to_le_bytes());
        packet.extend_from_slice(&packet_type.to_le_bytes());
        packet.extend_from_slice(payload_bytes);
        packet.extend_from_slice(&[0u8; 2]);

        self.stream.write_all(&packet)?;
        self.stream.flush()?;

        Ok(())
    }

    fn read_packet(&mut self) -> Result<RconResponse, TaskError> {
        let mut header = [0u8; 12];
        self.stream
            .read_exact(&mut header)
            .map_err(|e| TaskError::new("READ_HEADER", &e.to_string(), true))?;

        let length = i32::from_le_bytes([header[0], header[1], header[2], header[3]]);
        let id = i32::from_le_bytes([header[4], header[5], header[6], header[7]]);
        let packet_type = i32::from_le_bytes([header[8], header[9], header[10], header[11]]);

        let payload_length = (length - 10) as usize;
        let mut payload = vec![0u8; payload_length];
        self.stream
            .read_exact(&mut payload)
            .map_err(|e| TaskError::new("READ_PAYLOAD", &e.to_string(), true))?;

        let payload_str = String::from_utf8_lossy(&payload[..payload_length.saturating_sub(2)])
            .trim_end_matches('\0')
            .to_string();

        Ok(RconResponse {
            id,
            packet_type,
            payload: payload_str,
        })
    }
}

#[derive(Debug)]
struct RconResponse {
    id: i32,
    #[allow(dead_code)]
    packet_type: i32,
    payload: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rcon_client_requires_server() {
        // This test will fail without RCON server
        let result = RconClient::connect("localhost", 25575, "password");
        assert!(result.is_err());
    }
}
