use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub nats: NatsConfig,
    pub keto: KetoConfig,
    pub rate_limiting: RateLimitConfig,
    pub observability: ObservabilityConfig,
    pub pdf: PdfConfig,
    pub cdn: CdnConfig,
    pub file_storage: FileStorageConfig,
    pub pactum: PactumConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout_seconds: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3000,
            timeout_seconds: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub run_migrations: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://adeptus:password@localhost:5432/adeptus".to_string(),
            max_connections: 20,
            min_connections: 5,
            run_migrations: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsConfig {
    pub url: Option<String>,
    pub stream_name: String,
    pub max_age_days: u64,
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            url: None,
            stream_name: "ADEPTUS_EVENTS".to_string(),
            max_age_days: 30,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KetoConfig {
    pub read_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub global_limit: u32,
    pub per_user_limit: u32,
    pub per_ip_limit: u32,
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            global_limit: 10000,
            per_user_limit: 1000,
            per_ip_limit: 100,
            burst_size: 10,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub logging: LoggingConfig,
    pub tracing: TracingConfig,
    pub metrics: MetricsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "pretty".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    pub enabled: bool,
    pub otlp_endpoint: String,
    pub sampling_ratio: f64,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            otlp_endpoint: "http://localhost:4317".to_string(),
            sampling_ratio: 0.1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfConfig {
    pub wkhtmltopdf_path: String,
    pub temp_dir: String,
    pub generation_timeout_seconds: u64,
}

impl Default for PdfConfig {
    fn default() -> Self {
        Self {
            wkhtmltopdf_path: "wkhtmltopdf".to_string(),
            temp_dir: "/tmp/adeptus-pdf".to_string(),
            generation_timeout_seconds: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdnConfig {
    pub enabled: bool,
    pub base_url: String,
    pub api_key: String,
    pub bucket_name: String,
    pub upload_timeout_seconds: u64,
}

impl Default for CdnConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: String::new(),
            api_key: String::new(),
            bucket_name: String::new(),
            upload_timeout_seconds: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStorageConfig {
    pub upload_dir: String,
    pub max_file_size: u64,
}

impl Default for FileStorageConfig {
    fn default() -> Self {
        Self {
            upload_dir: "./uploads".to_string(),
            max_file_size: 104_857_600, // 100MB
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PactumConfig {
    pub graphql_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 3000);
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig::default();
        assert!(config.url.contains("adeptus"));
        assert_eq!(config.max_connections, 20);
    }

    #[test]
    fn test_nats_config_default() {
        let config = NatsConfig::default();
        assert!(config.url.is_none());
        assert_eq!(config.stream_name, "ADEPTUS_EVENTS");
    }

    #[test]
    fn test_pdf_config_default() {
        let config = PdfConfig::default();
        assert_eq!(config.wkhtmltopdf_path, "wkhtmltopdf");
        assert_eq!(config.generation_timeout_seconds, 30);
    }

    #[test]
    fn test_file_storage_config_default() {
        let config = FileStorageConfig::default();
        assert_eq!(config.max_file_size, 104_857_600);
    }
}
