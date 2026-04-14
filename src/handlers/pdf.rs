use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{Json, Response},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use crate::AppState;
use crate::config::PdfConfig;
use crate::error::AdeptusError;

static PDF_STYLES: &str = include_str!("../assets/pdf_styles.css");

#[derive(Deserialize)]
pub struct VersionQuery {
    pub current_version: Option<i32>,
}

#[derive(Serialize)]
pub struct VersionResponse {
    pub document_id: String,
    pub latest_revision: i32,
    pub is_outdated: bool,
}

pub async fn generate_pdf_handler(
    State(state): State<AppState>,
    Path(document_id): Path<Uuid>,
) -> Result<Response, AdeptusError> {
    let doc = state
        .repos
        .documents()
        .get_by_id(document_id)
        .await?
        .ok_or_else(|| AdeptusError::DocumentNotFound {
            id: document_id.to_string(),
        })?;

    let latest_rev = state
        .repos
        .document_revisions()
        .get_latest_revision_number(document_id)
        .await?;

    let pdf_bytes = generate_pdf_bytes(
        &doc.content,
        document_id,
        &doc.title,
        Some(latest_rev),
        &state.config.pdf,
    )
    .await?;

    let filename = format!("{}_v{}.pdf", doc.slug, latest_rev);

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        )
        .body(Body::from(pdf_bytes))
        .map_err(|e| AdeptusError::Internal {
            message: format!("Failed to build response: {e}"),
        })?;

    Ok(response)
}

pub async fn document_version_handler(
    State(state): State<AppState>,
    Path(document_id): Path<Uuid>,
    Query(query): Query<VersionQuery>,
) -> Result<Json<VersionResponse>, AdeptusError> {
    let _ = state
        .repos
        .documents()
        .get_by_id(document_id)
        .await?
        .ok_or_else(|| AdeptusError::DocumentNotFound {
            id: document_id.to_string(),
        })?;

    let latest_rev = state
        .repos
        .document_revisions()
        .get_latest_revision_number(document_id)
        .await?;

    let is_outdated = query.current_version.is_some_and(|cv| cv < latest_rev);

    Ok(Json(VersionResponse {
        document_id: document_id.to_string(),
        latest_revision: latest_rev,
        is_outdated,
    }))
}

pub async fn generate_pdf_bytes(
    markdoc_content: &str,
    document_id: Uuid,
    title: &str,
    version: Option<i32>,
    config: &PdfConfig,
) -> Result<Vec<u8>, AdeptusError> {
    let html = markdoc_to_html(markdoc_content, title, version);

    let temp_dir = PathBuf::from(&config.temp_dir);
    tokio::fs::create_dir_all(&temp_dir)
        .await
        .map_err(|e| AdeptusError::PdfGenerationError {
            message: format!("Failed to create temp directory: {e}"),
        })?;

    let input_file = temp_dir.join(format!("{}.html", document_id));
    let output_file = temp_dir.join(format!("{}.pdf", document_id));

    tokio::fs::write(&input_file, &html)
        .await
        .map_err(|e| AdeptusError::PdfGenerationError {
            message: format!("Failed to write HTML file: {e}"),
        })?;

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(config.generation_timeout_seconds),
        tokio::process::Command::new(&config.wkhtmltopdf_path)
            .args([
                "--page-size",
                "A4",
                "--margin-top",
                "20mm",
                "--margin-bottom",
                "20mm",
                "--margin-left",
                "15mm",
                "--margin-right",
                "15mm",
                "--encoding",
                "UTF-8",
                "--enable-local-file-access",
            ])
            .arg(input_file.to_str().unwrap_or_default())
            .arg(output_file.to_str().unwrap_or_default())
            .output(),
    )
    .await
    .map_err(|_| AdeptusError::PdfGenerationError {
        message: "PDF generation timed out".to_string(),
    })?
    .map_err(|e| AdeptusError::PdfGenerationError {
        message: format!("Failed to execute wkhtmltopdf: {e}"),
    })?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        // wkhtmltopdf often returns exit code 1 even on success with warnings
        if result.status.code() != Some(1) || !output_file.exists() {
            return Err(AdeptusError::PdfGenerationError {
                message: format!("wkhtmltopdf failed: {stderr}"),
            });
        }
    }

    let pdf_bytes =
        tokio::fs::read(&output_file)
            .await
            .map_err(|e| AdeptusError::PdfGenerationError {
                message: format!("Failed to read generated PDF: {e}"),
            })?;

    // Cleanup temp files
    let _ = tokio::fs::remove_file(&input_file).await;
    let _ = tokio::fs::remove_file(&output_file).await;

    Ok(pdf_bytes)
}

fn markdoc_to_html(content: &str, title: &str, version: Option<i32>) -> String {
    let body_html = simple_markdoc_conversion(content);
    let version_str = version.map(|v| format!(" (v{})", v)).unwrap_or_default();

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}{version_str}</title>
    <style>
{PDF_STYLES}
    </style>
</head>
<body>
    <div class="document-header">
        <h1>{title}</h1>
        <p class="version">Version: {version_display}</p>
    </div>
    <div class="document-content">
        {body_html}
    </div>
</body>
</html>"#,
        title = html_escape::encode_text(title),
        version_str = html_escape::encode_text(&version_str),
        version_display = version.unwrap_or(1),
        body_html = body_html,
    )
}

fn simple_markdoc_conversion(content: &str) -> String {
    let mut html = String::new();
    let mut in_code_block = false;
    let mut in_list = false;

    for line in content.lines() {
        if line.starts_with("```") {
            if in_code_block {
                html.push_str("</code></pre>\n");
                in_code_block = false;
            } else {
                if in_list {
                    html.push_str("</ul>\n");
                    in_list = false;
                }
                html.push_str("<pre><code>");
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            html.push_str(&html_escape::encode_text(line));
            html.push('\n');
            continue;
        }

        let trimmed = line.trim();

        if trimmed.is_empty() {
            if in_list {
                html.push_str("</ul>\n");
                in_list = false;
            }
            continue;
        }

        if let Some(heading) = trimmed.strip_prefix("######") {
            html.push_str(&format!(
                "<h6>{}</h6>\n",
                process_inline_formatting(heading.trim())
            ));
        } else if let Some(heading) = trimmed.strip_prefix("#####") {
            html.push_str(&format!(
                "<h5>{}</h5>\n",
                process_inline_formatting(heading.trim())
            ));
        } else if let Some(heading) = trimmed.strip_prefix("####") {
            html.push_str(&format!(
                "<h4>{}</h4>\n",
                process_inline_formatting(heading.trim())
            ));
        } else if let Some(heading) = trimmed.strip_prefix("###") {
            html.push_str(&format!(
                "<h3>{}</h3>\n",
                process_inline_formatting(heading.trim())
            ));
        } else if let Some(heading) = trimmed.strip_prefix("##") {
            html.push_str(&format!(
                "<h2>{}</h2>\n",
                process_inline_formatting(heading.trim())
            ));
        } else if let Some(heading) = trimmed.strip_prefix('#') {
            html.push_str(&format!(
                "<h1>{}</h1>\n",
                process_inline_formatting(heading.trim())
            ));
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            if !in_list {
                html.push_str("<ul>\n");
                in_list = true;
            }
            html.push_str(&format!(
                "<li>{}</li>\n",
                process_inline_formatting(&trimmed[2..])
            ));
        } else if trimmed.starts_with("---") || trimmed.starts_with("***") {
            html.push_str("<hr>\n");
        } else {
            if in_list {
                html.push_str("</ul>\n");
                in_list = false;
            }
            html.push_str(&format!("<p>{}</p>\n", process_inline_formatting(trimmed)));
        }
    }

    if in_code_block {
        html.push_str("</code></pre>\n");
    }
    if in_list {
        html.push_str("</ul>\n");
    }

    html
}

fn process_inline_formatting(text: &str) -> String {
    let escaped = html_escape::encode_text(text).to_string();

    // Bold: **text**
    let re_bold = regex::Regex::new(r"\*\*(.+?)\*\*").unwrap();
    let result = re_bold.replace_all(&escaped, "<strong>$1</strong>");

    // Italic: *text*
    let re_italic = regex::Regex::new(r"\*(.+?)\*").unwrap();
    let result = re_italic.replace_all(&result, "<em>$1</em>");

    // Inline code: `text`
    let re_code = regex::Regex::new(r"`(.+?)`").unwrap();
    let result = re_code.replace_all(&result, "<code>$1</code>");

    // Links: [text](url)
    let re_link = regex::Regex::new(r"\[(.+?)\]\((.+?)\)").unwrap();
    let result = re_link.replace_all(&result, r#"<a href="$2">$1</a>"#);

    result.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_markdoc_conversion_headings() {
        let content = "# Title\n## Subtitle\n### Section";
        let html = simple_markdoc_conversion(content);
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<h2>Subtitle</h2>"));
        assert!(html.contains("<h3>Section</h3>"));
    }

    #[test]
    fn test_simple_markdoc_conversion_lists() {
        let content = "- Item 1\n- Item 2\n- Item 3";
        let html = simple_markdoc_conversion(content);
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>Item 1</li>"));
        assert!(html.contains("<li>Item 2</li>"));
        assert!(html.contains("</ul>"));
    }

    #[test]
    fn test_simple_markdoc_conversion_code_block() {
        let content = "```\nlet x = 1;\n```";
        let html = simple_markdoc_conversion(content);
        assert!(html.contains("<pre><code>"));
        assert!(html.contains("let x = 1;"));
        assert!(html.contains("</code></pre>"));
    }

    #[test]
    fn test_process_inline_formatting() {
        assert!(process_inline_formatting("**bold**").contains("<strong>bold</strong>"));
        assert!(process_inline_formatting("`code`").contains("<code>code</code>"));
    }

    #[test]
    fn test_markdoc_to_html() {
        let html = markdoc_to_html("# Hello", "Test Doc", Some(3));
        assert!(html.contains("<title>Test Doc (v3)</title>"));
        assert!(html.contains("Version: 3"));
        assert!(html.contains("<h1>Hello</h1>"));
    }
}
