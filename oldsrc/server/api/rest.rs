//! REST API Interface
//!
//! This module provides REST API endpoints for server management.

use super::{ApiResponse, ServerStatus};
use crate::server::types::{HealthStatus, ServerMetrics};
use serde_json::Value;
use std::collections::HashMap;

/// REST API handler
pub struct RestApi {
    _placeholder: (),
}

impl RestApi {
    /// Create new REST API handler
    pub fn new() -> Self {
        Self { _placeholder: () }
    }

    /// Handle GET /status
    pub async fn get_status(&self) -> ApiResponse<ServerStatus> {
        let status = ServerStatus {
            version: env!("CARGO_PKG_VERSION").to_string(),
            health: HealthStatus::Healthy,
            metrics: ServerMetrics::default(),
        };
        ApiResponse::success(status)
    }

    /// Handle GET /health
    pub async fn get_health(&self) -> ApiResponse<HealthStatus> {
        ApiResponse::success(HealthStatus::Healthy)
    }

    /// Handle GET /metrics
    pub async fn get_metrics(&self) -> ApiResponse<ServerMetrics> {
        ApiResponse::success(ServerMetrics::default())
    }

    /// Handle POST /shutdown
    pub async fn post_shutdown(&self) -> ApiResponse<Value> {
        // TODO: Implement graceful shutdown
        ApiResponse::success(serde_json::json!({ "message": "Shutdown initiated" }))
    }

    /// Handle GET /config
    pub async fn get_config(&self) -> ApiResponse<HashMap<String, Value>> {
        // TODO: Return current configuration
        ApiResponse::success(HashMap::new())
    }
}

impl Default for RestApi {
    fn default() -> Self {
        Self::new()
    }
}
