//! Health alerting system.
//!
//! This module handles alerting when health issues are detected.

use super::{HealthSeverity, OverallHealth};
use crate::types::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Manages health alerts and notifications.
#[derive(Debug, Clone)]
pub struct AlertManager {
    alert_rules: Vec<AlertRule>,
    active_alerts: HashMap<String, ActiveAlert>,
    alert_history: Vec<AlertEvent>,
    max_history_size: usize,
}

impl AlertManager {
    /// Creates a new alert manager.
    pub fn new() -> Self {
        Self {
            alert_rules: Vec::new(),
            active_alerts: HashMap::new(),
            alert_history: Vec::new(),
            max_history_size: 1000,
        }
    }

    /// Adds an alert rule.
    pub fn add_rule(&mut self, rule: AlertRule) {
        self.alert_rules.push(rule);
    }

    /// Removes an alert rule by name.
    pub fn remove_rule(&mut self, name: &str) -> bool {
        let original_len = self.alert_rules.len();
        self.alert_rules.retain(|rule| rule.name != name);
        self.alert_rules.len() != original_len
    }

    /// Processes health status and triggers alerts if needed.
    pub async fn process_health_status(&mut self, health: &OverallHealth) -> Result<()> {
        let mut triggered_alerts = Vec::new();
        let mut resolved_alerts = Vec::new();

        // Check each rule against current health status
        for rule in &self.alert_rules {
            let should_alert = rule.evaluate(health);
            let alert_id = format!(
                "{}_{}",
                rule.name,
                rule.check_name.as_deref().unwrap_or("overall")
            );

            match (should_alert, self.active_alerts.contains_key(&alert_id)) {
                (true, false) => {
                    // New alert triggered
                    let alert = ActiveAlert {
                        rule: rule.clone(),
                        triggered_at: Instant::now(),
                        last_triggered: Instant::now(),
                        trigger_count: 1,
                    };

                    self.active_alerts.insert(alert_id.clone(), alert.clone());
                    triggered_alerts.push((alert_id, alert));
                }
                (true, true) => {
                    // Existing alert still active
                    if let Some(alert) = self.active_alerts.get_mut(&alert_id) {
                        alert.last_triggered = Instant::now();
                        alert.trigger_count += 1;
                    }
                }
                (false, true) => {
                    // Alert resolved
                    if let Some(alert) = self.active_alerts.remove(&alert_id) {
                        resolved_alerts.push((alert_id, alert));
                    }
                }
                (false, false) => {
                    // No alert, no change
                }
            }
        }

        // Process triggered alerts
        for (alert_id, alert) in triggered_alerts {
            self.trigger_alert(&alert_id, &alert, health).await?;
        }

        // Process resolved alerts
        for (alert_id, alert) in resolved_alerts {
            self.resolve_alert(&alert_id, &alert, health).await?;
        }

        Ok(())
    }

    /// Gets all active alerts.
    pub fn get_active_alerts(&self) -> &HashMap<String, ActiveAlert> {
        &self.active_alerts
    }

    /// Gets alert history.
    pub fn get_alert_history(&self) -> &[AlertEvent] {
        &self.alert_history
    }

    /// Clears resolved alerts from history.
    pub fn clear_resolved_alerts(&mut self) {
        self.alert_history
            .retain(|event| matches!(event.event_type, AlertEventType::Triggered));
    }

    async fn trigger_alert(
        &mut self,
        alert_id: &str,
        alert: &ActiveAlert,
        health: &OverallHealth,
    ) -> Result<()> {
        let event = AlertEvent {
            alert_id: alert_id.to_string(),
            event_type: AlertEventType::Triggered,
            severity: alert.rule.severity,
            message: self.format_alert_message(&alert.rule, health),
            timestamp: Instant::now(),
            metadata: self.collect_alert_metadata(alert, health),
        };

        self.add_to_history(event.clone());

        // Send notification based on alert severity and channel
        match &alert.rule.notification_channel {
            NotificationChannel::Log => {
                log::warn!("Health Alert [{}]: {}", alert.rule.severity, event.message);
            }
            NotificationChannel::Console => {
                eprintln!("Health Alert [{}]: {}", alert.rule.severity, event.message);
            }
            NotificationChannel::Email { recipients } => {
                // Placeholder for email notification
                log::info!(
                    "Would send email alert to {:?}: {}",
                    recipients,
                    event.message
                );
            }
            NotificationChannel::Webhook { url } => {
                // Placeholder for webhook notification
                log::info!("Would send webhook alert to {}: {}", url, event.message);
            }
        }

        Ok(())
    }

    async fn resolve_alert(
        &mut self,
        alert_id: &str,
        alert: &ActiveAlert,
        health: &OverallHealth,
    ) -> Result<()> {
        let event = AlertEvent {
            alert_id: alert_id.to_string(),
            event_type: AlertEventType::Resolved,
            severity: alert.rule.severity,
            message: format!("Alert resolved: {}", alert.rule.description),
            timestamp: Instant::now(),
            metadata: self.collect_alert_metadata(alert, health),
        };

        self.add_to_history(event.clone());

        // Send resolution notification
        match &alert.rule.notification_channel {
            NotificationChannel::Log => {
                log::info!(
                    "Health Alert Resolved [{}]: {}",
                    alert.rule.severity,
                    event.message
                );
            }
            NotificationChannel::Console => {
                println!(
                    "Health Alert Resolved [{}]: {}",
                    alert.rule.severity, event.message
                );
            }
            _ => {
                // Other channels can optionally send resolution notifications
            }
        }

        Ok(())
    }

    fn format_alert_message(&self, rule: &AlertRule, health: &OverallHealth) -> String {
        match &rule.check_name {
            Some(check_name) => {
                if let Some(check_result) = health.check_results.get(check_name) {
                    format!(
                        "{}: {} ({})",
                        rule.description, check_result.message, check_name
                    )
                } else {
                    format!("{}: Check not found ({})", rule.description, check_name)
                }
            }
            None => {
                format!("{}: {}", rule.description, health.message)
            }
        }
    }

    fn collect_alert_metadata(
        &self,
        alert: &ActiveAlert,
        health: &OverallHealth,
    ) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("rule_name".to_string(), alert.rule.name.clone());
        metadata.insert("severity".to_string(), format!("{:?}", alert.rule.severity));
        metadata.insert("trigger_count".to_string(), alert.trigger_count.to_string());
        metadata.insert(
            "duration".to_string(),
            format!("{:?}", alert.triggered_at.elapsed()),
        );
        metadata.insert(
            "health_severity".to_string(),
            format!("{:?}", health.severity),
        );

        if let Some(check_name) = &alert.rule.check_name {
            metadata.insert("check_name".to_string(), check_name.clone());
            if let Some(check_result) = health.check_results.get(check_name) {
                for (key, value) in &check_result.metadata {
                    metadata.insert(format!("check_{}", key), value.clone());
                }
            }
        }

        metadata
    }

    fn add_to_history(&mut self, event: AlertEvent) {
        self.alert_history.push(event);

        // Maintain maximum history size
        if self.alert_history.len() > self.max_history_size {
            self.alert_history.remove(0);
        }
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Alert rule definition.
#[derive(Debug, Clone)]
pub struct AlertRule {
    /// Unique name for the rule.
    pub name: String,
    /// Description of the alert.
    pub description: String,
    /// Severity level of the alert.
    pub severity: AlertSeverity,
    /// Condition to trigger the alert.
    pub condition: AlertCondition,
    /// Specific check name to monitor (None for overall health).
    pub check_name: Option<String>,
    /// Notification channel for the alert.
    pub notification_channel: NotificationChannel,
    /// Minimum time between repeated alerts.
    pub rate_limit: Duration,
}

impl AlertRule {
    /// Evaluates whether this rule should trigger an alert.
    pub fn evaluate(&self, health: &OverallHealth) -> bool {
        match &self.condition {
            AlertCondition::SeverityAtLeast(min_severity) => match &self.check_name {
                Some(check_name) => {
                    if let Some(check_result) = health.check_results.get(check_name) {
                        let check_severity = match check_result.status {
                            super::CheckStatus::Pass => HealthSeverity::Healthy,
                            super::CheckStatus::Warning => HealthSeverity::Warning,
                            super::CheckStatus::Fail => HealthSeverity::Critical,
                            super::CheckStatus::Error => HealthSeverity::Fatal,
                        };
                        check_severity >= *min_severity
                    } else {
                        false
                    }
                }
                None => health.severity >= *min_severity,
            },
            AlertCondition::CheckFailed => match &self.check_name {
                Some(check_name) => health
                    .check_results
                    .get(check_name)
                    .map(|result| {
                        matches!(
                            result.status,
                            super::CheckStatus::Fail | super::CheckStatus::Error
                        )
                    })
                    .unwrap_or(false),
                None => health.severity >= HealthSeverity::Critical,
            },
            AlertCondition::Custom(predicate) => predicate(health),
        }
    }
}

/// Alert condition types.
#[derive(Debug, Clone)]
pub enum AlertCondition {
    /// Alert when severity is at least the specified level.
    SeverityAtLeast(HealthSeverity),
    /// Alert when a specific check fails.
    CheckFailed,
    /// Custom condition function.
    Custom(fn(&OverallHealth) -> bool),
}

/// Alert severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    /// Informational alert.
    Info,
    /// Warning alert.
    Warning,
    /// Critical alert.
    Critical,
    /// Emergency alert.
    Emergency,
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertSeverity::Info => write!(f, "INFO"),
            AlertSeverity::Warning => write!(f, "WARNING"),
            AlertSeverity::Critical => write!(f, "CRITICAL"),
            AlertSeverity::Emergency => write!(f, "EMERGENCY"),
        }
    }
}

/// Notification channels for alerts.
#[derive(Debug, Clone)]
pub enum NotificationChannel {
    /// Log to application logs.
    Log,
    /// Print to console.
    Console,
    /// Send email notification.
    Email { recipients: Vec<String> },
    /// Send webhook notification.
    Webhook { url: String },
}

/// Active alert instance.
#[derive(Debug, Clone)]
pub struct ActiveAlert {
    /// Alert rule that triggered this alert.
    pub rule: AlertRule,
    /// When the alert was first triggered.
    pub triggered_at: Instant,
    /// When the alert was last triggered.
    pub last_triggered: Instant,
    /// Number of times this alert has been triggered.
    pub trigger_count: u32,
}

/// Alert event for history tracking.
#[derive(Debug, Clone)]
pub struct AlertEvent {
    /// Alert identifier.
    pub alert_id: String,
    /// Type of alert event.
    pub event_type: AlertEventType,
    /// Alert severity.
    pub severity: AlertSeverity,
    /// Alert message.
    pub message: String,
    /// Event timestamp.
    pub timestamp: Instant,
    /// Additional metadata.
    pub metadata: HashMap<String, String>,
}

/// Types of alert events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertEventType {
    /// Alert was triggered.
    Triggered,
    /// Alert was resolved.
    Resolved,
}
