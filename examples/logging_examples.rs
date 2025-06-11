//! RXServer Logging Examples
//!
//! This file demonstrates how to use the comprehensive logging system
//! in RXServer across different components.

use rxserver::logging::{
    // Core logging initialization
    init_logging,
    
    // Component-specific loggers
    ServerLogger, NetworkLogger, ProtocolLogger, ResourceLogger, PerformanceLogger,
    
    // Utility functions
    with_context, with_timing, with_timing_async, log_statistics,
    log_memory_usage_stats, log_rate_limited_warning, log_config_change,
    
    // Performance monitoring
    ConnectionStats, PerformanceMonitor, ScopedTimer,
    
    // Structured logging macros
    // These are automatically available when you use the crate
};
use rxserver::config::types::LoggingSettings;
use rxserver::{log_connection, log_x11_protocol, log_resource, log_performance, log_implementation};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn, error, debug};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize comprehensive logging
    let logging_config = LoggingSettings {
        level: rxserver::logging::types::LogLevel::Debug,
        file: Some(PathBuf::from("server.log")),
        stdout: true,
        max_file_size: 100,
        rotation_count: 5,
    };
    
    init_logging(&logging_config)?;
    info!("Logging system initialized");

    // 2. Server lifecycle logging
    demonstrate_server_logging().await;

    // 3. Network and connection logging
    demonstrate_network_logging().await;

    // 4. Protocol logging
    demonstrate_protocol_logging().await;

    // 5. Resource management logging
    demonstrate_resource_logging().await;

    // 6. Performance monitoring
    demonstrate_performance_logging().await;

    // 7. Utility functions
    demonstrate_utility_functions().await;

    // 8. Error handling and recovery
    demonstrate_error_logging().await;

    // 9. Statistics and monitoring
    demonstrate_statistics_logging().await;

    Ok(())
}

async fn demonstrate_server_logging() {
    info!("=== Server Lifecycle Logging ===");
    
    // Server startup
    ServerLogger::startup(0, "config.toml");
    
    // Configuration loading
    ServerLogger::config_loaded("/path/to/config.toml");
    
    // Server binding
    ServerLogger::bind_address("127.0.0.1:6000");
    
    // Server shutdown
    ServerLogger::shutdown();
}

async fn demonstrate_network_logging() {
    info!("=== Network Logging ===");
    
    let client_id = 123;
    let addr = "127.0.0.1:12345".parse().unwrap();
    
    // Connection events using structured macros
    log_connection!(accept, client_id, addr);
    log_connection!(disconnect, client_id, "client requested disconnection");
    
    // Using component logger
    NetworkLogger::listener_started("127.0.0.1:6000");
    NetworkLogger::connection_accepted(client_id, addr);
    NetworkLogger::connection_closed(client_id, "timeout");
}

async fn demonstrate_protocol_logging() {
    info!("=== Protocol Logging ===");
    
    let client_id = 123;
    
    // X11 protocol events using macros
    log_x11_protocol!(request, client_id, "CreateWindow");
    log_x11_protocol!(response, client_id, "CreateWindowResponse");
    log_x11_protocol!(error, client_id, 1, "BadWindow");
    
    // Using component logger
    ProtocolLogger::request_received(client_id, "MapWindow");
    ProtocolLogger::response_sent(client_id, "MapWindowResponse");
    ProtocolLogger::handshake_completed(client_id, 11, 0);
    ProtocolLogger::protocol_error(client_id, 2, "BadValue");
}

async fn demonstrate_resource_logging() {
    info!("=== Resource Management Logging ===");
    
    let client_id = 123;
    let window_id = 456;
    let pixmap_id = 789;
    
    // Resource events using macros
    log_resource!(create, "window", window_id, client_id);
    log_resource!(destroy, "pixmap", pixmap_id, client_id);
    log_resource!(leak, "font", 999, client_id);
    
    // Using component logger
    ResourceLogger::window_created(window_id, client_id);
    ResourceLogger::pixmap_destroyed(pixmap_id, client_id);
    ResourceLogger::resource_leak_detected("graphics_context", 555, client_id);
    ResourceLogger::cleanup_client_resources(client_id, 5);
}

async fn demonstrate_performance_logging() {
    info!("=== Performance Monitoring ===");
    
    // Performance metrics using macros
    log_performance!(timing, "request_processing", 15.5);
    log_performance!(memory, "client_buffers", 2048);
    log_performance!(counter, "requests_per_second", 125.0);
    
    // Using component logger
    PerformanceLogger::operation_timing("window_creation", 12.3);
    PerformanceLogger::memory_usage("pixmap_cache", 1024 * 1024);
    PerformanceLogger::request_rate(150.0);
    PerformanceLogger::frame_rate(60.0);
    PerformanceLogger::avg_response_time(8.5);
    
    // Timing with utility functions
    let result = with_timing("database_query", || {
        // Simulate some work
        std::thread::sleep(Duration::from_millis(50));
        "query result"
    });
    info!("Query result: {}", result);
    
    // Async timing
    let async_result = with_timing_async("async_operation", || async {
        sleep(Duration::from_millis(30)).await;
        "async result"
    }).await;
    info!("Async result: {}", async_result);
    
    // Scoped timer (automatically logs when dropped)
    {
        let _timer = ScopedTimer::new("scoped_operation");
        std::thread::sleep(Duration::from_millis(25));
        // Timer logs automatically when it goes out of scope
    }
}

async fn demonstrate_utility_functions() {
    info!("=== Utility Functions ===");
    
    // Context logging
    with_context("server_initialization", || {
        info!("Performing complex initialization");
        std::thread::sleep(Duration::from_millis(10));
    });
    
    // Statistics logging
    let stats = [
        ("total_connections", 42),
        ("active_connections", 8),
        ("requests_processed", 1337),
        ("errors_handled", 3),
    ];
    log_statistics("Server", &stats);
    
    // Configuration changes
    log_config_change("log_level", "info", "debug");
    log_config_change("max_clients", "100", "200");
    
    // Memory usage statistics
    log_memory_usage_stats("window_manager", 512 * 1024, 1024 * 1024);
    
    // Rate-limited warnings (useful for frequent errors)
    for i in 0..15 {
        log_rate_limited_warning("connection_timeout", 
            &format!("Connection timeout occurred (iteration {})", i));
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
}

async fn demonstrate_error_logging() {
    info!("=== Error Handling and Recovery ===");
    
    // Simulated error for demonstration
    let error = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Connection refused");
    
    // Retry logging
    for attempt in 1..=3 {
        warn!("Attempting operation (attempt {})", attempt);
        
        if attempt < 3 {
            use rxserver::logging::log_retry_warning;
            log_retry_warning("tcp_connect", attempt, 3, &error);
        } else {
            use rxserver::logging::log_recovery;
            log_recovery("tcp_connect", attempt);
            break;
        }
        
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn demonstrate_statistics_logging() {
    info!("=== Statistics and Monitoring ===");
    
    // Connection statistics
    let mut conn_stats = ConnectionStats::default();
    
    // Simulate some activity
    conn_stats.connection_opened();
    conn_stats.connection_opened();
    conn_stats.connection_opened();
    conn_stats.requests_processed = 150;
    conn_stats.responses_sent = 148;
    conn_stats.errors_sent = 2;
    conn_stats.update_response_time(12.5);
    conn_stats.update_response_time(8.3);
    conn_stats.update_response_time(15.7);
    conn_stats.bytes_sent = 1024 * 50;
    conn_stats.bytes_received = 1024 * 30;
    
    // Log detailed statistics
    conn_stats.log_stats();
    
    // Log summary
    conn_stats.log_summary();
    
    // Performance monitor
    let monitor = PerformanceMonitor::new();
    
    // Record some operations
    monitor.record_timing("request_parse", Duration::from_millis(5));
    monitor.record_timing("request_parse", Duration::from_millis(8));
    monitor.record_timing("response_build", Duration::from_millis(12));
    monitor.record_memory_usage("client_manager", 1024 * 512);
    monitor.record_memory_usage("resource_manager", 1024 * 256);
    
    // Force a report
    monitor.maybe_log_report(true);
}

async fn demonstrate_implementation_status() {
    info!("=== Implementation Status Logging ===");
    
    // Implementation status using macros
    log_implementation!(complete, "tcp_listener", "TCP connections working perfectly");
    log_implementation!(partial, "window_manager", "Basic window operations implemented");
    log_implementation!(todo, "unix_sockets", "Unix domain sockets need implementation");
    log_implementation!(missing, "hardware_accel", "GPU acceleration not yet started");
}
