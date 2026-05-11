//! Compression utilities for backup

use std::io::{Read, Write};

pub fn compress_zstd(input: &[u8], level: i32) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = zstd::Encoder::new(Vec::new(), level)?;
    encoder.write_all(input)?;
    encoder.finish()
}

pub fn decompress_zstd(input: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    zstd::decode_all(input)
}

pub fn compress_gzip(input: &[u8], level: flate2::Compression) -> Result<Vec<u8>, std::io::Error> {
    use flate2::write::GzEncoder;
    let mut encoder = GzEncoder::new(Vec::new(), level);
    encoder.write_all(input)?;
    encoder.finish()
}

pub fn decompress_gzip(input: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    use flate2::read::GzDecoder;
    let mut decoder = GzDecoder::new(input);
    let mut output = Vec::new();
    decoder.read_to_end(&mut output)?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zstd_roundtrip() {
        let data = b"Hello, World!";

        let compressed = compress_zstd(data, 3).unwrap();
        let decompressed = decompress_zstd(&compressed).unwrap();

        assert_eq!(data.as_slice(), decompressed.as_slice());
    }
}
