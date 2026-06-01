//! Archive creation — creates tar archives with zstd/gzip compression
//! for game server data directories.

use std::path::Path;
use sha2::{Sha256, Digest};

/// Compression format for backup archives
pub enum CompressionFormat {
    /// Zstandard compression with configurable level (1-22)
    Zstd(i32),
    /// Gzip compression with configurable level (1-9)
    Gzip(u32),
}

/// Create a tar archive of a container's data directory.
/// Compresses with zstd or gzip, returns (size_bytes, checksum_hex).
pub async fn create_container_backup(
    container_id: &str,
    data_path: &str,
    dest_path: &Path,
    compression: CompressionFormat,
) -> anyhow::Result<(u64, String)> {
    use tokio::process::Command;
    use tokio::io::AsyncWriteExt;

    tracing::info!(
        container_id = %container_id,
        dest = %dest_path.display(),
        "Creating container backup archive"
    );

    // Create parent directory
    if let Some(parent) = dest_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let compression_flag = match compression {
        CompressionFormat::Zstd(_) => "--zstd",
        CompressionFormat::Gzip(_) => "-z",
    };

    let output = Command::new("docker")
        .args([
            "exec", container_id,
            "tar", compression_flag, "-cf", "-",
            "-C", data_path, ".",
        ])
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to exec tar in container: {}", e))?;

    if !output.status.success() {
        // Fall back to podman if docker failed
        let podman_output = Command::new("podman")
            .args([
                "exec", container_id,
                "tar", compression_flag, "-cf", "-",
                "-C", data_path, ".",
            ])
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to exec tar in container (podman fallback): {}", e))?;

        if !podman_output.status.success() {
            let stderr = String::from_utf8_lossy(&podman_output.stderr);
            return Err(anyhow::anyhow!("Tar archive creation failed: {}", stderr));
        }

        let mut file = tokio::fs::File::create(dest_path).await?;
        file.write_all(&podman_output.stdout).await?;
        file.flush().await?;
        let data = &podman_output.stdout;
        let size = data.len() as u64;
        let checksum = calculate_checksum_bytes(data);
        tracing::info!(size_bytes = size, "Backup archive created via podman");
        Ok((size, checksum))
    } else {
        let mut file = tokio::fs::File::create(dest_path).await?;
        file.write_all(&output.stdout).await?;
        file.flush().await?;
        let data = &output.stdout;
        let size = data.len() as u64;
        let checksum = calculate_checksum_bytes(data);
        tracing::info!(size_bytes = size, "Backup archive created via docker");
        Ok((size, checksum))
    }
}

/// Calculate SHA-256 checksum of file at given path.
pub async fn calculate_checksum(path: &Path) -> anyhow::Result<String> {
    let data = tokio::fs::read(path).await?;
    Ok(calculate_checksum_bytes(&data))
}

fn calculate_checksum_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_deterministic() {
        let data = b"test backup data";
        let c1 = calculate_checksum_bytes(data);
        let c2 = calculate_checksum_bytes(data);
        assert_eq!(c1, c2);
        assert_eq!(c1.len(), 64);
    }
}
