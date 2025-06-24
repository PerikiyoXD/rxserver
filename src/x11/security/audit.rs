//! Security event auditing
//!
//! This module provides comprehensive security event logging and auditing
//! capabilities for the X11 server.

use std::collections::VecDeque;
use std::fmt;
use std::time::{Instant, SystemTime};

use crate::x11::protocol::types::ClientId;

/// Types of security events that can be audited
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityEventType {
    /// Client authentication attempt
    AuthenticationAttempt,
    /// Client authentication success
    AuthenticationSuccess,
    /// Client authentication failure
    AuthenticationFailure,
    /// Authorization check performed
    AuthorizationCheck,
    /// Authorization granted
    AuthorizationGranted,
    /// Authorization denied
    AuthorizationDenied,
    /// Resource limit violation
    ResourceLimitViolation,
    /// Security policy violation
    PolicyViolation,
    /// Privilege escalation attempt
    PrivilegeEscalation,
    /// Suspicious activity detected
    SuspiciousActivity,
    /// Client connection established
    ClientConnected,
    /// Client connection terminated
    ClientDisconnected,
    /// Configuration change
    ConfigurationChange,
}

/// Severity levels for security events
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Informational events
    Info,
    /// Warning events
    Warning,
    /// Error events
    Error,
    /// Critical security events
    Critical,
}

/// A security event that occurred in the system
#[derive(Debug, Clone)]
pub struct SecurityEvent {
    /// Unique identifier for this event
    pub id: u64,
    /// Type of security event
    pub event_type: SecurityEventType,
    /// Severity level
    pub severity: Severity,
    /// When the event occurred
    pub timestamp: SystemTime,
    /// Client involved in the event (if applicable)
    pub client_id: Option<ClientId>,
    /// Human-readable description
    pub description: String,
    /// Additional context data
    pub context: Vec<(String, String)>,
    /// Source component that generated the event
    pub source: String,
}

/// Configuration for the audit logger
#[derive(Debug, Clone)]
pub struct AuditConfig {
    /// Maximum number of events to keep in memory
    pub max_events: usize,
    /// Minimum severity level to log
    pub min_severity: Severity,
    /// Whether to log to system audit trail
    pub enable_system_audit: bool,
    /// Whether to log authentication events
    pub log_authentication: bool,
    /// Whether to log authorization events
    pub log_authorization: bool,
    /// Whether to log resource limit violations
    pub log_resource_limits: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            max_events: 10000,
            min_severity: Severity::Info,
            enable_system_audit: true,
            log_authentication: true,
            log_authorization: true,
            log_resource_limits: true,
        }
    }
}

/// Security event auditing logger
#[derive(Debug)]
pub struct AuditLogger {
    /// Configuration
    config: AuditConfig,
    /// In-memory event log (circular buffer)
    events: VecDeque<SecurityEvent>,
    /// Next event ID
    next_event_id: u64,
    /// Creation time for relative timestamps
    start_time: Instant,
}

impl AuditLogger {
    /// Create a new audit logger with default configuration
    pub fn new() -> Self {
        Self::with_config(AuditConfig::default())
    }

    /// Create a new audit logger with custom configuration
    pub fn with_config(config: AuditConfig) -> Self {
        Self {
            config,
            events: VecDeque::new(),
            next_event_id: 1,
            start_time: Instant::now(),
        }
    }

    /// Log a security event
    pub fn log_event(
        &mut self,
        event_type: SecurityEventType,
        severity: Severity,
        client_id: Option<ClientId>,
        description: String,
        source: String,
    ) {
        self.log_event_with_context(
            event_type,
            severity,
            client_id,
            description,
            source,
            Vec::new(),
        );
    }

    /// Log a security event with additional context
    pub fn log_event_with_context(
        &mut self,
        event_type: SecurityEventType,
        severity: Severity,
        client_id: Option<ClientId>,
        description: String,
        source: String,
        context: Vec<(String, String)>,
    ) {
        // Check if we should log this event based on severity
        if severity < self.config.min_severity {
            return;
        }

        // Check if this event type is enabled
        if !self.should_log_event_type(&event_type) {
            return;
        }

        let event = SecurityEvent {
            id: self.next_event_id,
            event_type,
            severity,
            timestamp: SystemTime::now(),
            client_id,
            description,
            context,
            source,
        };

        self.next_event_id += 1;

        // Add to in-memory log
        self.events.push_back(event.clone());

        // Maintain size limit
        while self.events.len() > self.config.max_events {
            self.events.pop_front();
        }

        // Log to system audit trail if enabled
        if self.config.enable_system_audit {
            self.write_to_system_audit(&event);
        }
    }

    /// Log an authentication attempt
    pub fn log_authentication_attempt(&mut self, client_id: ClientId, method: &str) {
        self.log_event_with_context(
            SecurityEventType::AuthenticationAttempt,
            Severity::Info,
            Some(client_id),
            format!("Client authentication attempt using method: {}", method),
            "authentication".to_string(),
            vec![("method".to_string(), method.to_string())],
        );
    }

    /// Log successful authentication
    pub fn log_authentication_success(&mut self, client_id: ClientId, method: &str) {
        self.log_event_with_context(
            SecurityEventType::AuthenticationSuccess,
            Severity::Info,
            Some(client_id),
            format!("Client authenticated successfully using method: {}", method),
            "authentication".to_string(),
            vec![("method".to_string(), method.to_string())],
        );
    }

    /// Log failed authentication
    pub fn log_authentication_failure(&mut self, client_id: ClientId, method: &str, reason: &str) {
        self.log_event_with_context(
            SecurityEventType::AuthenticationFailure,
            Severity::Warning,
            Some(client_id),
            format!("Client authentication failed: {}", reason),
            "authentication".to_string(),
            vec![
                ("method".to_string(), method.to_string()),
                ("reason".to_string(), reason.to_string()),
            ],
        );
    }

    /// Log an authorization check
    pub fn log_authorization_check(&mut self, client_id: ClientId, operation: &str) {
        self.log_event_with_context(
            SecurityEventType::AuthorizationCheck,
            Severity::Info,
            Some(client_id),
            format!("Authorization check for operation: {}", operation),
            "authorization".to_string(),
            vec![("operation".to_string(), operation.to_string())],
        );
    }

    /// Log authorization granted
    pub fn log_authorization_granted(&mut self, client_id: ClientId, operation: &str) {
        self.log_event_with_context(
            SecurityEventType::AuthorizationGranted,
            Severity::Info,
            Some(client_id),
            format!("Authorization granted for operation: {}", operation),
            "authorization".to_string(),
            vec![("operation".to_string(), operation.to_string())],
        );
    }

    /// Log authorization denied
    pub fn log_authorization_denied(&mut self, client_id: ClientId, operation: &str, reason: &str) {
        self.log_event_with_context(
            SecurityEventType::AuthorizationDenied,
            Severity::Warning,
            Some(client_id),
            format!(
                "Authorization denied for operation {}: {}",
                operation, reason
            ),
            "authorization".to_string(),
            vec![
                ("operation".to_string(), operation.to_string()),
                ("reason".to_string(), reason.to_string()),
            ],
        );
    }

    /// Log a resource limit violation
    pub fn log_resource_limit_violation(
        &mut self,
        client_id: ClientId,
        resource: &str,
        limit: &str,
    ) {
        self.log_event_with_context(
            SecurityEventType::ResourceLimitViolation,
            Severity::Error,
            Some(client_id),
            format!("Resource limit violation: {} exceeded {}", resource, limit),
            "resource_limits".to_string(),
            vec![
                ("resource".to_string(), resource.to_string()),
                ("limit".to_string(), limit.to_string()),
            ],
        );
    }

    /// Get recent security events
    pub fn get_recent_events(&self, count: usize) -> Vec<SecurityEvent> {
        self.events.iter().rev().take(count).cloned().collect()
    }

    /// Get events by severity
    pub fn get_events_by_severity(&self, severity: Severity) -> Vec<SecurityEvent> {
        self.events
            .iter()
            .filter(|event| event.severity >= severity)
            .cloned()
            .collect()
    }

    /// Get events for a specific client
    pub fn get_client_events(&self, client_id: ClientId) -> Vec<SecurityEvent> {
        self.events
            .iter()
            .filter(|event| event.client_id == Some(client_id))
            .cloned()
            .collect()
    }

    /// Clear all events
    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    /// Get current configuration
    pub fn get_config(&self) -> &AuditConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: AuditConfig) {
        self.config = config;
    }

    /// Check if an event type should be logged based on configuration
    fn should_log_event_type(&self, event_type: &SecurityEventType) -> bool {
        match event_type {
            SecurityEventType::AuthenticationAttempt
            | SecurityEventType::AuthenticationSuccess
            | SecurityEventType::AuthenticationFailure => self.config.log_authentication,
            SecurityEventType::AuthorizationCheck
            | SecurityEventType::AuthorizationGranted
            | SecurityEventType::AuthorizationDenied => self.config.log_authorization,
            SecurityEventType::ResourceLimitViolation => self.config.log_resource_limits,
            _ => true, // Log other event types by default
        }
    }

    /// Write event to system audit trail (placeholder implementation)
    fn write_to_system_audit(&self, event: &SecurityEvent) {
        // In a real implementation, this would write to the system's audit log
        // For now, we'll just use eprintln for demonstration
        eprintln!(
            "AUDIT: [{}] {:?} - {}",
            event
                .timestamp
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            event.severity,
            event.description
        );
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Info => write!(f, "INFO"),
            Severity::Warning => write!(f, "WARN"),
            Severity::Error => write!(f, "ERROR"),
            Severity::Critical => write!(f, "CRITICAL"),
        }
    }
}

impl fmt::Display for SecurityEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityEventType::AuthenticationAttempt => write!(f, "AUTH_ATTEMPT"),
            SecurityEventType::AuthenticationSuccess => write!(f, "AUTH_SUCCESS"),
            SecurityEventType::AuthenticationFailure => write!(f, "AUTH_FAILURE"),
            SecurityEventType::AuthorizationCheck => write!(f, "AUTHZ_CHECK"),
            SecurityEventType::AuthorizationGranted => write!(f, "AUTHZ_GRANTED"),
            SecurityEventType::AuthorizationDenied => write!(f, "AUTHZ_DENIED"),
            SecurityEventType::ResourceLimitViolation => write!(f, "RESOURCE_LIMIT"),
            SecurityEventType::PolicyViolation => write!(f, "POLICY_VIOLATION"),
            SecurityEventType::PrivilegeEscalation => write!(f, "PRIVILEGE_ESCALATION"),
            SecurityEventType::SuspiciousActivity => write!(f, "SUSPICIOUS_ACTIVITY"),
            SecurityEventType::ClientConnected => write!(f, "CLIENT_CONNECTED"),
            SecurityEventType::ClientDisconnected => write!(f, "CLIENT_DISCONNECTED"),
            SecurityEventType::ConfigurationChange => write!(f, "CONFIG_CHANGE"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_logger_creation() {
        let logger = AuditLogger::new();
        assert_eq!(logger.events.len(), 0);
        assert_eq!(logger.next_event_id, 1);
    }

    #[test]
    fn test_log_event() {
        let mut logger = AuditLogger::new();

        logger.log_event(
            SecurityEventType::AuthenticationSuccess,
            Severity::Info,
            Some(1),
            "Test event".to_string(),
            "test".to_string(),
        );

        assert_eq!(logger.events.len(), 1);
        let event = &logger.events[0];
        assert_eq!(event.id, 1);
        assert_eq!(event.event_type, SecurityEventType::AuthenticationSuccess);
        assert_eq!(event.severity, Severity::Info);
        assert_eq!(event.client_id, Some(1));
        assert_eq!(event.description, "Test event");
        assert_eq!(event.source, "test");
    }

    #[test]
    fn test_severity_filtering() {
        let config = AuditConfig {
            min_severity: Severity::Warning,
            ..Default::default()
        };
        let mut logger = AuditLogger::with_config(config);

        // Info event should be filtered out
        logger.log_event(
            SecurityEventType::AuthenticationSuccess,
            Severity::Info,
            Some(1),
            "Info event".to_string(),
            "test".to_string(),
        );

        // Warning event should be logged
        logger.log_event(
            SecurityEventType::AuthenticationFailure,
            Severity::Warning,
            Some(1),
            "Warning event".to_string(),
            "test".to_string(),
        );

        assert_eq!(logger.events.len(), 1);
        assert_eq!(logger.events[0].severity, Severity::Warning);
    }

    #[test]
    fn test_max_events_limit() {
        let config = AuditConfig {
            max_events: 2,
            ..Default::default()
        };
        let mut logger = AuditLogger::with_config(config);

        // Add 3 events
        for i in 1..=3 {
            logger.log_event(
                SecurityEventType::AuthenticationSuccess,
                Severity::Info,
                Some(i),
                format!("Event {}", i),
                "test".to_string(),
            );
        }

        // Should only keep the last 2 events
        assert_eq!(logger.events.len(), 2);
        assert_eq!(logger.events[0].description, "Event 2");
        assert_eq!(logger.events[1].description, "Event 3");
    }

    #[test]
    fn test_get_recent_events() {
        let mut logger = AuditLogger::new();

        // Add some events
        for i in 1..=5 {
            logger.log_event(
                SecurityEventType::AuthenticationSuccess,
                Severity::Info,
                Some(i),
                format!("Event {}", i),
                "test".to_string(),
            );
        }

        let recent = logger.get_recent_events(3);
        assert_eq!(recent.len(), 3);
        // Should be in reverse order (most recent first)
        assert_eq!(recent[0].description, "Event 5");
        assert_eq!(recent[1].description, "Event 4");
        assert_eq!(recent[2].description, "Event 3");
    }

    #[test]
    fn test_get_events_by_severity() {
        let mut logger = AuditLogger::new();

        logger.log_event(
            SecurityEventType::AuthenticationSuccess,
            Severity::Info,
            Some(1),
            "Info event".to_string(),
            "test".to_string(),
        );

        logger.log_event(
            SecurityEventType::AuthenticationFailure,
            Severity::Error,
            Some(2),
            "Error event".to_string(),
            "test".to_string(),
        );

        let error_events = logger.get_events_by_severity(Severity::Error);
        assert_eq!(error_events.len(), 1);
        assert_eq!(error_events[0].severity, Severity::Error);
    }

    #[test]
    fn test_authentication_logging() {
        let mut logger = AuditLogger::new();

        logger.log_authentication_attempt(1, "password");
        logger.log_authentication_success(1, "password");
        logger.log_authentication_failure(2, "password", "invalid credentials");

        assert_eq!(logger.events.len(), 3);
        assert_eq!(
            logger.events[0].event_type,
            SecurityEventType::AuthenticationAttempt
        );
        assert_eq!(
            logger.events[1].event_type,
            SecurityEventType::AuthenticationSuccess
        );
        assert_eq!(
            logger.events[2].event_type,
            SecurityEventType::AuthenticationFailure
        );
    }
}
