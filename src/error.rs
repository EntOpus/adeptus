use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

pub type AdeptusResult<T> = Result<T, AdeptusError>;

#[derive(Debug, Clone, Serialize, thiserror::Error)]
#[serde(tag = "error_type")]
pub enum AdeptusError {
    #[error("Document not found: {id}")]
    DocumentNotFound { id: String },

    #[error("Document category not found: {id}")]
    DocumentCategoryNotFound { id: String },

    #[error("Document file not found: {id}")]
    DocumentFileNotFound { id: String },

    #[error("Glossary entry not found: {id}")]
    GlossaryEntryNotFound { id: String },

    #[error("Glossary category not found: {id}")]
    GlossaryCategoryNotFound { id: String },

    #[error("Invalid document status: {status}")]
    InvalidDocumentStatus { status: String },

    #[error("Validation error: {message}")]
    ValidationError { message: String },

    #[error("Insufficient permissions: {required}")]
    InsufficientPermissions { required: String },

    #[error("Consent required: {message}")]
    ConsentRequired { message: String },

    #[error("PDF generation error: {message}")]
    PdfGenerationError { message: String },

    #[error("File upload error: {message}")]
    FileUploadError { message: String },

    #[error("File too large: {size} bytes exceeds limit of {limit} bytes")]
    FileTooLarge { size: u64, limit: u64 },

    #[error("Database error: {message}")]
    DatabaseError { message: String },

    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl AdeptusError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AdeptusError::DocumentNotFound { .. } => StatusCode::NOT_FOUND,
            AdeptusError::DocumentCategoryNotFound { .. } => StatusCode::NOT_FOUND,
            AdeptusError::DocumentFileNotFound { .. } => StatusCode::NOT_FOUND,
            AdeptusError::GlossaryEntryNotFound { .. } => StatusCode::NOT_FOUND,
            AdeptusError::GlossaryCategoryNotFound { .. } => StatusCode::NOT_FOUND,
            AdeptusError::InvalidDocumentStatus { .. } => StatusCode::BAD_REQUEST,
            AdeptusError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            AdeptusError::InsufficientPermissions { .. } => StatusCode::FORBIDDEN,
            AdeptusError::ConsentRequired { .. } => StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS,
            AdeptusError::PdfGenerationError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            AdeptusError::FileUploadError { .. } => StatusCode::BAD_REQUEST,
            AdeptusError::FileTooLarge { .. } => StatusCode::BAD_REQUEST,
            AdeptusError::DatabaseError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            AdeptusError::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn is_retriable(&self) -> bool {
        matches!(self, AdeptusError::DatabaseError { .. })
    }

    pub fn is_client_error(&self) -> bool {
        self.status_code().is_client_error()
    }
}

impl IntoResponse for AdeptusError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = serde_json::json!({
            "error": self.to_string(),
            "status": status.as_u16(),
        });
        (status, axum::Json(body)).into_response()
    }
}

impl From<anyhow::Error> for AdeptusError {
    fn from(err: anyhow::Error) -> Self {
        AdeptusError::Internal {
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for AdeptusError {
    fn from(err: serde_json::Error) -> Self {
        AdeptusError::Internal {
            message: format!("Serialization error: {}", err),
        }
    }
}

impl From<sqlx::Error> for AdeptusError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AdeptusError::DocumentNotFound {
                id: "unknown".to_string(),
            },
            _ => AdeptusError::DatabaseError {
                message: err.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_codes() {
        assert_eq!(
            AdeptusError::DocumentNotFound { id: "x".into() }.status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            AdeptusError::GlossaryEntryNotFound { id: "x".into() }.status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            AdeptusError::ValidationError {
                message: "x".into()
            }
            .status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AdeptusError::InsufficientPermissions {
                required: "x".into()
            }
            .status_code(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            AdeptusError::DatabaseError {
                message: "x".into()
            }
            .status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_is_retriable() {
        assert!(
            AdeptusError::DatabaseError {
                message: "x".into()
            }
            .is_retriable()
        );
        assert!(
            !AdeptusError::Internal {
                message: "x".into()
            }
            .is_retriable()
        );
    }

    #[test]
    fn test_is_client_error() {
        assert!(AdeptusError::DocumentNotFound { id: "x".into() }.is_client_error());
        assert!(
            !AdeptusError::DatabaseError {
                message: "x".into()
            }
            .is_client_error()
        );
    }

    #[test]
    fn test_from_anyhow() {
        let err: AdeptusError = anyhow::anyhow!("boom").into();
        assert!(matches!(err, AdeptusError::Internal { .. }));
    }

    #[test]
    fn test_from_serde_json() {
        let json_err = serde_json::from_str::<String>("not-json").unwrap_err();
        let err: AdeptusError = json_err.into();
        assert!(matches!(err, AdeptusError::Internal { .. }));
    }
}
