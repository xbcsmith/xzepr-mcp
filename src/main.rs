//! XZepr MCP Server - Main Entry Point
//!
//! This is the main entry point for the XZepr MCP server application.
//! It handles CLI argument parsing, configuration loading, observability
//! initialization, and server startup with graceful shutdown.
//!
//! # Example Usage
//!
//! ```bash
//! # Start server with default configuration
//! xzepr-mcp
//!
//! # Start server with custom config file
//! xzepr-mcp --config /path/to/config.yaml
//!
//! # Start server with environment variable overrides
//! XZEPR_MCP_SERVER_PORT=8081 xzepr-mcp
//!
//! # Start server with specific log level
//! xzepr-mcp --log-level debug
//! ```

use clap::Parser;
use std::path::PathBuf;
use tracing::{error, info};
use xzepr_mcp::config::{SecurityCheck, Settings};

/// XZepr MCP Server - Model Context Protocol adapter for XZepr Event Tracking
#[derive(Parser, Debug)]
#[command(name = "xzepr-mcp")]
#[command(author = "XZepr Team")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "MCP server for XZepr Event Tracking System", long_about = None)]
struct Cli {
    /// Path to configuration file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Server host to bind to
    #[arg(long, env = "XZEPR_MCP_SERVER_HOST")]
    host: Option<String>,

    /// Server port to bind to
    #[arg(short, long, env = "XZEPR_MCP_SERVER_PORT")]
    port: Option<u16>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, env = "XZEPR_MCP_LOG_LEVEL", default_value = "info")]
    log_level: String,

    /// Enable JSON log format
    #[arg(long, env = "XZEPR_MCP_LOG_JSON")]
    log_json: bool,

    /// OIDC provider URL
    #[arg(long, env = "XZEPR_MCP_OIDC_PROVIDER_URL")]
    oidc_provider_url: Option<String>,

    /// JWT issuer
    #[arg(long, env = "XZEPR_MCP_JWT_ISSUER")]
    jwt_issuer: Option<String>,

    /// XZepr API base URL
    #[arg(long, env = "XZEPR_MCP_XZEPR_BASE_URL")]
    xzepr_base_url: Option<String>,

    /// Enable metrics endpoint
    #[arg(long, env = "XZEPR_MCP_ENABLE_METRICS")]
    enable_metrics: bool,

    /// Metrics port
    #[arg(long, env = "XZEPR_MCP_METRICS_PORT")]
    metrics_port: Option<u16>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize basic logging for startup
    init_startup_logging(&cli.log_level, cli.log_json);

    info!(
        version = env!("CARGO_PKG_VERSION"),
        "Starting XZepr MCP Server"
    );

    // Load configuration
    let config_path = cli.config.as_ref().map(|p| p.to_str().unwrap());
    let mut settings = match Settings::load(config_path) {
        Ok(s) => s,
        Err(e) => {
            error!(error = %e, "Failed to load configuration");
            return Err(e.into());
        }
    };

    info!("Configuration loaded successfully");

    // Apply CLI overrides
    apply_cli_overrides(&mut settings, &cli);

    // Validate configuration
    if let Err(e) = settings.validate() {
        error!(error = %e, "Configuration validation failed");
        return Err(e.into());
    }

    info!("Configuration validated successfully");

    // Initialize observability (logging, metrics, tracing)
    // TODO: Uncomment when observability module is complete
    // if let Err(e) = xzepr_mcp::observability::init(&settings) {
    //     error!(error = %e, "Failed to initialize observability");
    //     return Err(e.into());
    // }

    info!(
        host = %settings.server.host,
        port = %settings.server.port,
        "Observability initialized"
    );

    // Display configuration summary
    display_config_summary(&settings);

    // Start MCP server
    info!("Starting MCP server...");

    // TODO: Uncomment when MCP server is fully implemented
    // let server = xzepr_mcp::mcp::McpServer::new(settings).await?;
    // server.start().await?;

    // Placeholder message until server is complete
    info!("MCP server would start here. Server implementation needs completion.");
    info!("Phase 1 foundation is complete. Phase 2 MCP protocol implementation in progress.");

    // Keep the application running until Ctrl+C
    info!("Press Ctrl+C to stop");
    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received");

    Ok(())
}

/// Initialize basic logging for application startup
fn init_startup_logging(level: &str, json: bool) {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(level))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = if json {
        fmt::layer().json().with_filter(filter).boxed()
    } else {
        fmt::layer()
            .with_target(true)
            .with_level(true)
            .with_filter(filter)
            .boxed()
    };

    tracing_subscriber::registry().with(fmt_layer).init();
}

/// Apply CLI argument overrides to settings
fn apply_cli_overrides(settings: &mut Settings, cli: &Cli) {
    if let Some(ref host) = cli.host {
        settings.server.host = host.clone();
    }

    if let Some(port) = cli.port {
        settings.server.port = port;
    }

    if let Some(ref oidc_url) = cli.oidc_provider_url {
        settings.auth.oidc_provider_url = oidc_url.clone();
    }

    if let Some(ref issuer) = cli.jwt_issuer {
        settings.auth.jwt_issuer = issuer.clone();
    }

    if let Some(ref base_url) = cli.xzepr_base_url {
        settings.xzepr.base_url = base_url.clone();
    }

    if cli.enable_metrics {
        settings.observability.enable_metrics = true;
    }

    if let Some(port) = cli.metrics_port {
        settings.observability.metrics_port = port;
    }

    // Override log level from CLI
    settings.observability.log_level = cli.log_level.clone();
}

/// Display configuration summary at startup
fn display_config_summary(settings: &Settings) {
    info!("=== Configuration Summary ===");
    info!("Server:");
    info!("  Host: {}", settings.server.host);
    info!("  Port: {}", settings.server.port);
    info!(
        "  Request Timeout: {}s",
        settings.server.request_timeout_secs
    );
    info!(
        "  Max Payload Size: {} bytes",
        settings.server.max_payload_size
    );
    info!("  CORS Enabled: {}", settings.server.enable_cors);

    info!("XZepr API:");
    info!("  Base URL: {}", settings.xzepr.base_url);
    info!("  Timeout: {}s", settings.xzepr.timeout_secs);
    info!("  Max Retries: {}", settings.xzepr.max_retries);
    info!(
        "  Circuit Breaker: {}",
        if settings.xzepr.enable_circuit_breaker {
            "enabled"
        } else {
            "disabled"
        }
    );

    info!("Authentication:");
    info!("  OIDC Provider: {}", settings.auth.oidc_provider_url);
    info!("  JWT Issuer: {}", settings.auth.jwt_issuer);
    info!("  JWT Audience: {}", settings.auth.jwt_audience);
    info!("  JWKS Cache TTL: {}s", settings.auth.jwks_cache_ttl_secs);
    info!(
        "  JWT Validation: {}",
        if settings.auth.enable_jwt_validation {
            "enabled"
        } else {
            "disabled"
        }
    );

    info!("Rate Limiting:");
    info!(
        "  Enabled: {}",
        if settings.rate_limit.enabled {
            "yes"
        } else {
            "no"
        }
    );
    if settings.rate_limit.enabled {
        info!(
            "  Global: {}/min",
            settings.rate_limit.global_requests_per_minute
        );
        info!(
            "  Per User: {}/min",
            settings.rate_limit.per_user_requests_per_minute
        );
        info!(
            "  Read Operations: {}/min",
            settings.rate_limit.per_tool_limits.read_operations
        );
        info!(
            "  Write Operations: {}/min",
            settings.rate_limit.per_tool_limits.write_operations
        );
        info!(
            "  Search Operations: {}/min",
            settings.rate_limit.per_tool_limits.search_operations
        );
    }

    info!("Observability:");
    info!(
        "  Tracing: {}",
        if settings.observability.enable_tracing {
            "enabled"
        } else {
            "disabled"
        }
    );
    info!("  Log Level: {}", settings.observability.log_level);
    info!(
        "  Metrics: {}",
        if settings.observability.enable_metrics {
            "enabled"
        } else {
            "disabled"
        }
    );
    if settings.observability.enable_metrics {
        info!("  Metrics Port: {}", settings.observability.metrics_port);
    }
    info!(
        "  Audit Logging: {}",
        if settings.observability.enable_audit_logging {
            "enabled"
        } else {
            "disabled"
        }
    );

    info!("Security:");
    info!("  Max JSON Depth: {}", settings.security.max_json_depth);
    info!(
        "  SQL Injection Detection: {}",
        settings
            .security
            .detection
            .is_enabled(SecurityCheck::SqlInjection)
    );
    info!(
        "  XSS Detection: {}",
        settings.security.detection.is_enabled(SecurityCheck::Xss)
    );
    info!(
        "  Path Traversal Detection: {}",
        settings
            .security
            .detection
            .is_enabled(SecurityCheck::PathTraversal)
    );
    info!("=============================");
}
