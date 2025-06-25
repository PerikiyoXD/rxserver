// //! Server API Interfaces
// //!
// //! This module provides various API interfaces for server management and monitoring.
//
// pub mod cli;
// pub mod rest;
// pub mod rpc;
// pub mod websocket;
//
// use crate::server::types::ServerMetrics;
//
// /// Common API response
// #[derive(Debug, serde::Serialize)]
// pub struct ApiResponse<T> {
//     /// Success status
//     pub success: bool,
//     /// Response data
//     pub data: Option<T>,
//     /// Error message if any
//     pub error: Option<String>,
// }
//
// impl<T> ApiResponse<T> {
//     /// Create a successful response
//     pub fn success(data: T) -> Self {
//         Self {
//             success: true,
//             data: Some(data),
//             error: None,
//         }
//     }
//
//     /// Create an error response
//     pub fn error(message: String) -> Self {
//         Self {
//             success: false,
//             data: None,
//             error: Some(message),
//         }
//     }
// }
//
// /// Server status information
// #[derive(Debug, serde::Serialize)]
// pub struct ServerStatus {
//     /// Server version
//     pub version: String,
//     /// Server health status
//     pub health: crate::server::types::HealthStatus,
//     /// Server metrics
//     pub metrics: ServerMetrics,
// }
