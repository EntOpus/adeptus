use axum::{
    body::Body,
    extract::{Extension, Multipart, Path, State},
    http::{StatusCode, header},
    response::{Json, Response},
};
use serde::Serialize;
use std::path::PathBuf;
use uuid::Uuid;

use crate::AppState;
use crate::error::AdeptusError;
use crate::middleware::SubjectContext;
use crate::platform_events::{EventResource, PlatformEvent};

#[derive(Serialize)]
pub struct UploadResponse {
    pub files: Vec<UploadedFile>,
}

#[derive(Serialize)]
pub struct UploadedFile {
    pub id: String,
    pub filename: String,
    pub mime_type: String,
    pub file_size: i64,
}

const ALLOWED_EXTENSIONS: &[&str] = &[
    "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", "csv", "md", "rtf", "odt", "ods",
    "odp", "png", "jpg", "jpeg", "gif", "svg", "webp", "bmp", "ico", "mp4", "webm", "mp3", "wav",
    "zip", "gz", "tar", "json", "xml", "html", "css", "js",
];

pub async fn upload_file(
    State(state): State<AppState>,
    Extension(subject): Extension<SubjectContext>,
    Path(document_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, AdeptusError> {
    // Verify document exists
    let _doc = state
        .repos
        .documents()
        .get_by_id(document_id)
        .await?
        .ok_or_else(|| AdeptusError::DocumentNotFound {
            id: document_id.to_string(),
        })?;

    let upload_dir = PathBuf::from(&state.config.file_storage.upload_dir);
    let doc_dir = upload_dir.join(document_id.to_string());
    tokio::fs::create_dir_all(&doc_dir)
        .await
        .map_err(|e| AdeptusError::FileUploadError {
            message: format!("Failed to create upload directory: {e}"),
        })?;

    let mut uploaded_files = Vec::new();
    let max_size = state.config.file_storage.max_file_size;
    let subject_id: Uuid =
        subject
            .subject_id
            .parse()
            .map_err(|_| AdeptusError::ValidationError {
                message: "Invalid subject ID".to_string(),
            })?;

    while let Some(field) =
        multipart
            .next_field()
            .await
            .map_err(|e| AdeptusError::FileUploadError {
                message: format!("Failed to read multipart field: {e}"),
            })?
    {
        let original_filename = field.file_name().unwrap_or("unnamed").to_string();

        // Validate extension
        let extension = original_filename
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_lowercase();
        if !ALLOWED_EXTENSIONS.contains(&extension.as_str()) {
            return Err(AdeptusError::ValidationError {
                message: format!("File extension '{extension}' is not allowed"),
            });
        }

        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        let data = field
            .bytes()
            .await
            .map_err(|e| AdeptusError::FileUploadError {
                message: format!("Failed to read file data: {e}"),
            })?;

        let file_size = data.len() as u64;
        if file_size > max_size {
            return Err(AdeptusError::FileTooLarge {
                size: file_size,
                limit: max_size,
            });
        }

        let stored_filename = format!("{}_{}", Uuid::new_v4(), original_filename);
        let file_path = doc_dir.join(&stored_filename);

        tokio::fs::write(&file_path, &data)
            .await
            .map_err(|e| AdeptusError::FileUploadError {
                message: format!("Failed to write file: {e}"),
            })?;

        let file_path_str = file_path.to_string_lossy().to_string();
        let file_record = state
            .repos
            .document_files()
            .create(
                document_id,
                &stored_filename,
                &original_filename,
                &file_path_str,
                None,
                &content_type,
                data.len() as i64,
                subject_id,
            )
            .await?;

        // Fire-and-forget audit + event
        let repos = state.repos.clone();
        let events = state.events.clone();
        let file_id = file_record.id;
        let actor_id = subject_id;
        let ip = subject.ip_address.clone();
        let ua = subject.user_agent.clone();
        let fname = original_filename.clone();
        tokio::spawn(async move {
            if let Err(e) = repos
                .audit()
                .create_audit_log(
                    "document_file",
                    file_id,
                    "file_uploaded",
                    actor_id,
                    ip.as_deref(),
                    ua.as_deref(),
                    Some(serde_json::json!({"filename": fname, "document_id": document_id.to_string()})),
                )
                .await
            {
                tracing::warn!("Failed to create audit log: {e}");
            }
            events
                .publish(&PlatformEvent::new(
                    "adeptus.file.uploaded",
                    &actor_id.to_string(),
                    EventResource {
                        resource_type: "document_file".to_string(),
                        resource_id: file_id.to_string(),
                        resource_name: Some(fname),
                        resource_url: None,
                    },
                    serde_json::json!({"document_id": document_id.to_string()}),
                    None,
                ))
                .await;
        });

        uploaded_files.push(UploadedFile {
            id: file_record.id.to_string(),
            filename: file_record.original_filename,
            mime_type: file_record.mime_type,
            file_size: file_record.file_size,
        });
    }

    if uploaded_files.is_empty() {
        return Err(AdeptusError::ValidationError {
            message: "No files provided".to_string(),
        });
    }

    Ok(Json(UploadResponse {
        files: uploaded_files,
    }))
}

pub async fn download_file(
    State(state): State<AppState>,
    Path(file_id): Path<Uuid>,
) -> Result<Response, AdeptusError> {
    let file = state
        .repos
        .document_files()
        .get_by_id(file_id)
        .await?
        .ok_or_else(|| AdeptusError::DocumentFileNotFound {
            id: file_id.to_string(),
        })?;

    let data = tokio::fs::read(&file.file_path).await.map_err(|e| {
        tracing::error!("Failed to read file from disk: {e}");
        AdeptusError::Internal {
            message: format!("Failed to read file: {e}"),
        }
    })?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, &file.mime_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", file.original_filename),
        )
        .body(Body::from(data))
        .map_err(|e| AdeptusError::Internal {
            message: format!("Failed to build response: {e}"),
        })?;

    Ok(response)
}
