//! Upload — direct-to-storage upload for backup archives
//!
//! Supports S3-compatible storage (AWS S3, Cloudflare R2, MinIO, DO Spaces)
//! and local filesystem storage.

use std::path::Path;

/// Upload a backup file to S3-compatible storage.
/// Uses rusoto_s3 with configurable endpoint and static credentials.
/// Returns the storage path (s3://bucket/key).
pub async fn upload_to_s3_with_config(
    endpoint: &str,
    bucket: &str,
    region: &str,
    access_key: &str,
    secret_key: &str,
    server_id: &str,
    file_name: &str,
    file_path: &Path,
) -> anyhow::Result<String> {
    use rusoto_core::{Region, credential::StaticProvider, HttpClient};
    use rusoto_s3::{S3, S3Client, PutObjectRequest};

    let credentials = StaticProvider::new(
        access_key.to_string(),
        secret_key.to_string(),
        None, None,
    );

    let s3_region = if region.is_empty() {
        Region::Custom {
            name: "auto".to_string(),
            endpoint: endpoint.to_string(),
        }
    } else {
        Region::Custom {
            name: region.to_string(),
            endpoint: endpoint.to_string(),
        }
    };

    let client = S3Client::new_with(HttpClient::new()?, credentials, s3_region);

    let file_data = tokio::fs::read(file_path).await
        .map_err(|e| anyhow::anyhow!("Failed to read backup file for upload: {}", e))?;

    let key = format!("{}/{}", server_id, file_name);

    let request = PutObjectRequest {
        bucket: bucket.to_string(),
        key: key.clone(),
        body: Some(file_data.into()),
        content_type: Some("application/zstd".to_string()),
        ..Default::default()
    };

    client.put_object(request).await
        .map_err(|e| anyhow::anyhow!("S3 upload failed: {}", e))?;

    tracing::info!(bucket = %bucket, key = %key, "Backup uploaded to S3-compatible storage");
    Ok(format!("s3://{}/{}", bucket, key))
}

/// Move/copy a backup file to the local storage directory.
/// Returns the absolute path as storage path.
pub async fn upload_to_local(
    source_path: &Path,
    storage_base: &Path,
    server_id: &str,
    file_name: &str,
) -> anyhow::Result<String> {
    let dest_dir = storage_base.join(server_id);
    tokio::fs::create_dir_all(&dest_dir).await?;

    let dest_path = dest_dir.join(file_name);

    tokio::fs::copy(source_path, &dest_path).await
        .map_err(|e| anyhow::anyhow!("Failed to copy backup to local storage: {}", e))?;

    let path_str = dest_path.to_string_lossy().to_string();
    tracing::info!(path = %path_str, "Backup stored locally");
    Ok(path_str)
}
