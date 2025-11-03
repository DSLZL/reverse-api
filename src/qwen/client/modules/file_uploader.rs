use super::auth::AuthManager;
use super::constants::{build_json_headers, BASE_URL};
use crate::qwen::error::{QwenError, Result};
use crate::qwen::models::{FileMeta, FileObject, QwenFile, StsTokenRequest, StsTokenResponse};
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

pub struct FileUploader {
    client: rquest::Client,
    auth: Arc<AuthManager>,
}

impl FileUploader {
    pub fn new(client: rquest::Client, auth: Arc<AuthManager>) -> Self {
        Self { client, auth }
    }

    fn get_file_info(extension: &str) -> (&'static str, &'static str, &'static str, &'static str) {
        match extension {
            "jpg" | "jpeg" => ("image", "vision", "image", "image/jpeg"),
            "png" => ("image", "vision", "image", "image/png"),
            "gif" => ("image", "vision", "image", "image/gif"),
            "webp" => ("image", "vision", "image", "image/webp"),
            "bmp" => ("image", "vision", "image", "image/bmp"),

            "mp4" => ("video", "video", "file", "video/mp4"),
            "avi" => ("video", "video", "file", "video/x-msvideo"),
            "mov" => ("video", "video", "file", "video/quicktime"),
            "mkv" => ("video", "video", "file", "video/x-matroska"),

            "mp3" => ("audio", "audio", "file", "audio/mpeg"),
            "wav" => ("audio", "audio", "file", "audio/wav"),
            "m4a" => ("audio", "audio", "file", "audio/mp4"),
            "flac" => ("audio", "audio", "file", "audio/flac"),

            "txt" => ("file", "document", "file", "text/plain"),
            "md" => ("file", "document", "file", "text/markdown"),
            "pdf" => ("file", "document", "file", "application/pdf"),
            "doc" => ("file", "document", "file", "application/msword"),
            "docx" => (
                "file",
                "document",
                "file",
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            ),
            "csv" => ("file", "document", "file", "text/csv"),
            "xls" => ("file", "document", "file", "application/vnd.ms-excel"),
            "xlsx" => (
                "file",
                "document",
                "file",
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            ),

            _ => ("file", "document", "file", "application/octet-stream"),
        }
    }

    pub async fn upload_file(&self, file_path: &str, user_id: String) -> Result<QwenFile> {
        let path = Path::new(file_path);

        if !path.exists() {
            return Err(QwenError::ApiError(format!(
                "File not found: {}",
                file_path
            )));
        }

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| QwenError::ApiError("Invalid filename".to_string()))?
            .to_string();

        let file_data = tokio::fs::read(file_path).await?;
        let filesize = file_data.len();

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let (filetype, file_class, show_type, content_type) = Self::get_file_info(&extension);
        let token = self.auth.get_token().await?;

        let sts_url = format!("{}/api/v2/files/getstsToken", BASE_URL);
        let headers = build_json_headers(Some(&token));

        let sts_request = StsTokenRequest {
            filename: filename.clone(),
            filesize,
            filetype: filetype.to_string(),
        };

        let sts_response = self
            .client
            .post(&sts_url)
            .headers(headers.clone())
            .json(&sts_request)
            .send()
            .await?;

        let sts_text = sts_response.text().await?;
        let sts_data: StsTokenResponse = serde_json::from_str(&sts_text).map_err(|e| {
            QwenError::ApiError(format!(
                "Failed to parse STS response: {} - {}",
                e, sts_text
            ))
        })?;

        if !sts_data.success {
            return Err(QwenError::ApiError(format!(
                "Failed to get STS token: {}",
                sts_text
            )));
        }

        self.upload_to_oss(&sts_data, &file_data, content_type)
            .await?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let qwen_file = QwenFile {
            r#type: filetype.to_string(),
            file: FileObject {
                created_at: timestamp,
                data: serde_json::json!({}),
                filename: filename.clone(),
                hash: None,
                id: sts_data.data.file_id.clone(),
                user_id,
                meta: FileMeta {
                    name: filename.clone(),
                    size: filesize,
                    content_type: content_type.to_string(),
                },
                update_at: timestamp,
            },
            id: sts_data.data.file_id,
            url: sts_data.data.file_url,
            name: filename,
            collection_name: String::new(),
            progress: 0,
            status: "uploaded".to_string(),
            green_net: if filetype == "video" {
                "greening"
            } else {
                "success"
            }
            .to_string(),
            size: filesize,
            error: String::new(),
            item_id: Uuid::new_v4().to_string(),
            file_type: content_type.to_string(),
            show_type: show_type.to_string(),
            file_class: file_class.to_string(),
            upload_task_id: Uuid::new_v4().to_string(),
        };

        Ok(qwen_file)
    }

    async fn upload_to_oss(
        &self,
        sts_data: &StsTokenResponse,
        file_data: &[u8],
        content_type: &str,
    ) -> Result<()> {
        use hmac::{Hmac, Mac};
        use sha2::{Digest, Sha256};
        type HmacSha256 = Hmac<Sha256>;

        let oss_url = format!(
            "https://{}.{}/{}",
            sts_data.data.bucketname, sts_data.data.endpoint, sts_data.data.file_path
        );

        let oss_date = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
        let date_short = &oss_date[0..8];
        let region = sts_data
            .data
            .region
            .strip_prefix("oss-")
            .unwrap_or(&sts_data.data.region);

        let credential = format!(
            "{}/{}/{}/oss/aliyun_v4_request",
            sts_data.data.access_key_id, date_short, region
        );

        let canonical_uri = format!("/{}/{}", sts_data.data.bucketname, sts_data.data.file_path);
        let canonical_headers = format!(
            "content-type:{}\nx-oss-content-sha256:UNSIGNED-PAYLOAD\nx-oss-date:{}\nx-oss-security-token:{}\nx-oss-user-agent:aliyun-sdk-js/6.23.0 Chrome 142.0.0.0 on OS X 10.15.7 64-bit\n",
            content_type,
            oss_date,
            sts_data.data.security_token
        );

        let canonical_request = format!(
            "PUT\n{}\n\n{}\n\nUNSIGNED-PAYLOAD",
            canonical_uri, canonical_headers
        );

        let mut hasher = Sha256::new();
        hasher.update(canonical_request.as_bytes());
        let request_hash = hex::encode(hasher.finalize());

        let string_to_sign = format!(
            "OSS4-HMAC-SHA256\n{}\n{}/{}/oss/aliyun_v4_request\n{}",
            oss_date, date_short, region, request_hash
        );

        let k_date = HmacSha256::new_from_slice(
            format!("aliyun_v4{}", sts_data.data.access_key_secret).as_bytes(),
        )
        .unwrap()
        .chain_update(date_short.as_bytes())
        .finalize()
        .into_bytes();

        let k_region = HmacSha256::new_from_slice(&k_date)
            .unwrap()
            .chain_update(region.as_bytes())
            .finalize()
            .into_bytes();

        let k_service = HmacSha256::new_from_slice(&k_region)
            .unwrap()
            .chain_update(b"oss")
            .finalize()
            .into_bytes();

        let k_signing = HmacSha256::new_from_slice(&k_service)
            .unwrap()
            .chain_update(b"aliyun_v4_request")
            .finalize()
            .into_bytes();

        let signature = HmacSha256::new_from_slice(&k_signing)
            .unwrap()
            .chain_update(string_to_sign.as_bytes())
            .finalize();
        let signature_hex = hex::encode(signature.into_bytes());

        let authorization = format!(
            "OSS4-HMAC-SHA256 Credential={},Signature={}",
            credential, signature_hex
        );

        let mut oss_headers = rquest::header::HeaderMap::new();
        oss_headers.insert("authorization", authorization.parse().unwrap());
        oss_headers.insert("content-type", content_type.parse().unwrap());
        oss_headers.insert("x-oss-date", oss_date.parse().unwrap());
        oss_headers.insert(
            "x-oss-security-token",
            sts_data.data.security_token.parse().unwrap(),
        );
        oss_headers.insert("x-oss-content-sha256", "UNSIGNED-PAYLOAD".parse().unwrap());
        oss_headers.insert(
            "x-oss-user-agent",
            "aliyun-sdk-js/6.23.0 Chrome 142.0.0.0 on OS X 10.15.7 64-bit"
                .parse()
                .unwrap(),
        );

        let oss_response = self
            .client
            .put(&oss_url)
            .headers(oss_headers)
            .body(file_data.to_vec())
            .send()
            .await?;

        let status = oss_response.status();
        if !status.is_success() {
            let error_text = oss_response.text().await?;
            return Err(QwenError::ApiError(format!(
                "OSS upload failed ({}): {}",
                status, error_text
            )));
        }

        Ok(())
    }
}
