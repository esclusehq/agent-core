//! Protocol version and negotiation

pub const PROTOCOL_VERSION: u32 = 1;

pub fn negotiate_version(agent: u32, backend: u32) -> u32 {
    agent.min(backend)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_negotiation() {
        assert_eq!(negotiate_version(1, 2), 1);
        assert_eq!(negotiate_version(2, 1), 1);
        assert_eq!(negotiate_version(1, 1), 1);
    }
}
