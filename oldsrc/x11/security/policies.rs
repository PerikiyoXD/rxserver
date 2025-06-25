//! Security policy enforcement
//!
//! This module provides security policy definition, management, and enforcement
//! for the X11 server.

use std::collections::HashMap;
use std::fmt;

use super::SecurityError;
use crate::x11::protocol::types::ClientId;

/// Types of security policies
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PolicyType {
    /// Access control policies
    AccessControl,
    /// Resource usage policies
    ResourceUsage,
    /// Authentication policies
    Authentication,
    /// Authorization policies
    Authorization,
    /// Network security policies
    Network,
    /// Audit policies
    Audit,
    /// General server policies
    General,
}

/// Policy enforcement actions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyAction {
    /// Allow the operation
    Allow,
    /// Deny the operation
    Deny,
    /// Log the operation but allow it
    LogAndAllow,
    /// Log the operation and deny it
    LogAndDeny,
    /// Require additional authentication
    RequireAuth,
    /// Apply rate limiting
    RateLimit {
        max_requests: u32,
        window_seconds: u32,
    },
    /// Quarantine the client
    Quarantine,
}

/// A security policy rule
#[derive(Debug, Clone)]
pub struct PolicyRule {
    /// Unique identifier for this rule
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what this rule does
    pub description: String,
    /// Policy type this rule belongs to
    pub policy_type: PolicyType,
    /// Conditions that must be met for this rule to apply
    pub conditions: Vec<PolicyCondition>,
    /// Action to take when this rule matches
    pub action: PolicyAction,
    /// Priority (higher numbers take precedence)
    pub priority: u32,
    /// Whether this rule is currently enabled
    pub enabled: bool,
}

/// Conditions for policy rule matching
#[derive(Debug, Clone, PartialEq)]
pub enum PolicyCondition {
    /// Client ID matches
    ClientId(ClientId),
    /// Client IP address matches pattern
    ClientIp(String),
    /// Operation type matches
    Operation(String),
    /// Resource type matches
    ResourceType(String),
    /// Time of day range
    TimeRange { start_hour: u8, end_hour: u8 },
    /// Day of week
    DayOfWeek(u8), // 0 = Sunday, 6 = Saturday
    /// Authentication method used
    AuthMethod(String),
    /// Client has specific permission
    HasPermission(String),
    /// Resource count exceeds threshold
    ResourceCountExceeds { resource_type: String, count: u32 },
    /// Request rate exceeds threshold
    RateExceeds { requests_per_second: f64 },
    /// Custom condition (extensible)
    Custom { name: String, value: String },
}

/// A complete security policy
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    /// Unique identifier for this policy
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of this policy
    pub description: String,
    /// Version of this policy
    pub version: String,
    /// Rules that make up this policy
    pub rules: Vec<PolicyRule>,
    /// Whether this policy is currently active
    pub active: bool,
}

/// Policy evaluation context
#[derive(Debug, Clone)]
pub struct PolicyContext {
    /// Client performing the operation
    pub client_id: Option<ClientId>,
    /// Client IP address
    pub client_ip: Option<String>,
    /// Operation being performed
    pub operation: String,
    /// Resource type involved
    pub resource_type: Option<String>,
    /// Authentication method used
    pub auth_method: Option<String>,
    /// Client permissions
    pub permissions: Vec<String>,
    /// Current resource counts
    pub resource_counts: HashMap<String, u32>,
    /// Current request rate
    pub request_rate: f64,
    /// Additional context data
    pub context: HashMap<String, String>,
}

/// Result of policy evaluation
#[derive(Debug, Clone)]
pub struct PolicyEvaluationResult {
    /// Action to take
    pub action: PolicyAction,
    /// Rules that matched
    pub matched_rules: Vec<String>,
    /// Reason for the decision
    pub reason: String,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Security policy manager
#[derive(Debug)]
pub struct PolicyManager {
    /// Active policies
    policies: HashMap<String, SecurityPolicy>,
    /// Default policy for unmatched cases
    default_action: PolicyAction,
    /// Policy evaluation statistics
    evaluation_stats: PolicyStats,
}

/// Policy evaluation statistics
#[derive(Debug, Default)]
struct PolicyStats {
    /// Total evaluations performed
    total_evaluations: u64,
    /// Total rules evaluated
    total_rule_evaluations: u64,
    /// Policy cache hits
    cache_hits: u64,
    /// Policy cache misses
    cache_misses: u64,
}

impl PolicyManager {
    /// Create a new policy manager
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
            default_action: PolicyAction::Allow,
            evaluation_stats: PolicyStats::default(),
        }
    }

    /// Add a security policy
    pub fn add_policy(&mut self, policy: SecurityPolicy) -> Result<(), SecurityError> {
        if policy.rules.is_empty() {
            return Err(SecurityError::InvalidPolicy);
        }

        self.policies.insert(policy.id.clone(), policy);
        Ok(())
    }

    /// Remove a security policy
    pub fn remove_policy(&mut self, policy_id: &str) -> Result<SecurityPolicy, SecurityError> {
        self.policies
            .remove(policy_id)
            .ok_or(SecurityError::InvalidPolicy)
    }

    /// Get a security policy by ID
    pub fn get_policy(&self, policy_id: &str) -> Option<&SecurityPolicy> {
        self.policies.get(policy_id)
    }

    /// Get all active policies
    pub fn get_active_policies(&self) -> Vec<&SecurityPolicy> {
        self.policies
            .values()
            .filter(|policy| policy.active)
            .collect()
    }

    /// Enable a policy
    pub fn enable_policy(&mut self, policy_id: &str) -> Result<(), SecurityError> {
        if let Some(policy) = self.policies.get_mut(policy_id) {
            policy.active = true;
            Ok(())
        } else {
            Err(SecurityError::InvalidPolicy)
        }
    }

    /// Disable a policy
    pub fn disable_policy(&mut self, policy_id: &str) -> Result<(), SecurityError> {
        if let Some(policy) = self.policies.get_mut(policy_id) {
            policy.active = false;
            Ok(())
        } else {
            Err(SecurityError::InvalidPolicy)
        }
    }

    /// Evaluate policies for a given context
    pub fn evaluate_policies(&mut self, context: &PolicyContext) -> PolicyEvaluationResult {
        self.evaluation_stats.total_evaluations += 1;

        let mut matched_rules = Vec::new();
        let mut highest_priority_action = self.default_action.clone();
        let mut highest_priority = 0;
        let mut reasons = Vec::new();

        // Collect all rules from active policies
        let mut all_rules: Vec<&PolicyRule> = Vec::new();
        for policy in self.policies.values() {
            if policy.active {
                for rule in &policy.rules {
                    if rule.enabled {
                        all_rules.push(rule);
                    }
                }
            }
        }

        // Sort rules by priority (highest first)
        all_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Evaluate rules in priority order
        for rule in all_rules {
            self.evaluation_stats.total_rule_evaluations += 1;

            if self.evaluate_rule(rule, context) {
                matched_rules.push(rule.id.clone());

                // Use the highest priority rule that matches
                if rule.priority >= highest_priority {
                    highest_priority = rule.priority;
                    highest_priority_action = rule.action.clone();
                    reasons.push(format!("Rule '{}' matched", rule.name));
                }
            }
        }

        let reason = if reasons.is_empty() {
            "No matching rules, using default policy".to_string()
        } else {
            reasons.join("; ")
        };

        PolicyEvaluationResult {
            action: highest_priority_action,
            matched_rules,
            reason,
            metadata: HashMap::new(),
        }
    }

    /// Evaluate a single rule against context
    fn evaluate_rule(&self, rule: &PolicyRule, context: &PolicyContext) -> bool {
        // All conditions must be true for the rule to match
        for condition in &rule.conditions {
            if !self.evaluate_condition(condition, context) {
                return false;
            }
        }
        true
    }

    /// Evaluate a single condition
    fn evaluate_condition(&self, condition: &PolicyCondition, context: &PolicyContext) -> bool {
        match condition {
            PolicyCondition::ClientId(id) => context.client_id == Some(*id),
            PolicyCondition::ClientIp(pattern) => {
                if let Some(ip) = &context.client_ip {
                    // Simple pattern matching (in real implementation, use proper IP matching)
                    ip.contains(pattern)
                } else {
                    false
                }
            }
            PolicyCondition::Operation(op) => context.operation == *op,
            PolicyCondition::ResourceType(resource_type) => {
                context.resource_type.as_ref() == Some(resource_type)
            }
            PolicyCondition::TimeRange {
                start_hour,
                end_hour,
            } => {
                // In real implementation, get current time
                // For now, always return true
                start_hour <= end_hour
            }
            PolicyCondition::DayOfWeek(_day) => {
                // In real implementation, check current day of week
                true
            }
            PolicyCondition::AuthMethod(method) => context.auth_method.as_ref() == Some(method),
            PolicyCondition::HasPermission(permission) => context.permissions.contains(permission),
            PolicyCondition::ResourceCountExceeds {
                resource_type,
                count,
            } => context.resource_counts.get(resource_type).unwrap_or(&0) > count,
            PolicyCondition::RateExceeds {
                requests_per_second,
            } => context.request_rate > *requests_per_second,
            PolicyCondition::Custom { name: _, value: _ } => {
                // Custom conditions would be evaluated by plugins
                false
            }
        }
    }

    /// Set the default action for unmatched cases
    pub fn set_default_action(&mut self, action: PolicyAction) {
        self.default_action = action;
    }

    /// Get policy evaluation statistics
    pub fn get_stats(&self) -> &PolicyStats {
        &self.evaluation_stats
    }

    /// Clear policy evaluation statistics
    pub fn clear_stats(&mut self) {
        self.evaluation_stats = PolicyStats::default();
    }

    /// Validate a policy before adding it
    pub fn validate_policy(policy: &SecurityPolicy) -> Result<(), SecurityError> {
        if policy.id.is_empty() || policy.name.is_empty() {
            return Err(SecurityError::InvalidPolicy);
        }

        if policy.rules.is_empty() {
            return Err(SecurityError::InvalidPolicy);
        }

        for rule in &policy.rules {
            Self::validate_rule(rule)?;
        }

        Ok(())
    }

    /// Validate a policy rule
    fn validate_rule(rule: &PolicyRule) -> Result<(), SecurityError> {
        if rule.id.is_empty() || rule.name.is_empty() {
            return Err(SecurityError::InvalidPolicy);
        }

        if rule.conditions.is_empty() {
            return Err(SecurityError::InvalidPolicy);
        }

        // Validate conditions
        for condition in &rule.conditions {
            Self::validate_condition(condition)?;
        }

        Ok(())
    }

    /// Validate a policy condition
    fn validate_condition(condition: &PolicyCondition) -> Result<(), SecurityError> {
        match condition {
            PolicyCondition::TimeRange {
                start_hour,
                end_hour,
            } => {
                if *start_hour > 23 || *end_hour > 23 {
                    return Err(SecurityError::InvalidPolicy);
                }
            }
            PolicyCondition::DayOfWeek(day) => {
                if *day > 6 {
                    return Err(SecurityError::InvalidPolicy);
                }
            }
            PolicyCondition::RateExceeds {
                requests_per_second,
            } => {
                if *requests_per_second < 0.0 {
                    return Err(SecurityError::InvalidPolicy);
                }
            }
            _ => {} // Other conditions are valid by construction
        }
        Ok(())
    }

    /// Create a default security policy
    pub fn create_default_policy() -> SecurityPolicy {
        SecurityPolicy {
            id: "default".to_string(),
            name: "Default Security Policy".to_string(),
            description: "Basic security policy with common rules".to_string(),
            version: "1.0".to_string(),
            rules: vec![
                PolicyRule {
                    id: "rate_limit".to_string(),
                    name: "Basic Rate Limiting".to_string(),
                    description: "Limit request rate to prevent DoS attacks".to_string(),
                    policy_type: PolicyType::General,
                    conditions: vec![PolicyCondition::RateExceeds {
                        requests_per_second: 100.0,
                    }],
                    action: PolicyAction::RateLimit {
                        max_requests: 100,
                        window_seconds: 1,
                    },
                    priority: 100,
                    enabled: true,
                },
                PolicyRule {
                    id: "resource_limit".to_string(),
                    name: "Resource Limit Enforcement".to_string(),
                    description: "Prevent excessive resource allocation".to_string(),
                    policy_type: PolicyType::ResourceUsage,
                    conditions: vec![PolicyCondition::ResourceCountExceeds {
                        resource_type: "window".to_string(),
                        count: 1000,
                    }],
                    action: PolicyAction::Deny,
                    priority: 200,
                    enabled: true,
                },
            ],
            active: true,
        }
    }
}

impl Default for PolicyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for PolicyAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PolicyAction::Allow => write!(f, "ALLOW"),
            PolicyAction::Deny => write!(f, "DENY"),
            PolicyAction::LogAndAllow => write!(f, "LOG_AND_ALLOW"),
            PolicyAction::LogAndDeny => write!(f, "LOG_AND_DENY"),
            PolicyAction::RequireAuth => write!(f, "REQUIRE_AUTH"),
            PolicyAction::RateLimit {
                max_requests,
                window_seconds,
            } => {
                write!(f, "RATE_LIMIT({}/{}s)", max_requests, window_seconds)
            }
            PolicyAction::Quarantine => write!(f, "QUARANTINE"),
        }
    }
}

impl fmt::Display for PolicyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PolicyType::AccessControl => write!(f, "ACCESS_CONTROL"),
            PolicyType::ResourceUsage => write!(f, "RESOURCE_USAGE"),
            PolicyType::Authentication => write!(f, "AUTHENTICATION"),
            PolicyType::Authorization => write!(f, "AUTHORIZATION"),
            PolicyType::Network => write!(f, "NETWORK"),
            PolicyType::Audit => write!(f, "AUDIT"),
            PolicyType::General => write!(f, "GENERAL"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_manager_creation() {
        let manager = PolicyManager::new();
        assert_eq!(manager.policies.len(), 0);
        assert!(matches!(manager.default_action, PolicyAction::Allow));
    }

    #[test]
    fn test_add_policy() {
        let mut manager = PolicyManager::new();
        let policy = PolicyManager::create_default_policy();

        assert!(manager.add_policy(policy).is_ok());
        assert_eq!(manager.policies.len(), 1);
        assert!(manager.get_policy("default").is_some());
    }

    #[test]
    fn test_policy_validation() {
        let policy = PolicyManager::create_default_policy();
        assert!(PolicyManager::validate_policy(&policy).is_ok());

        // Test invalid policy (empty rules)
        let invalid_policy = SecurityPolicy {
            id: "invalid".to_string(),
            name: "Invalid Policy".to_string(),
            description: "Policy with no rules".to_string(),
            version: "1.0".to_string(),
            rules: vec![],
            active: true,
        };
        assert!(PolicyManager::validate_policy(&invalid_policy).is_err());
    }

    #[test]
    fn test_policy_evaluation() {
        let mut manager = PolicyManager::new();
        let policy = PolicyManager::create_default_policy();
        manager.add_policy(policy).unwrap();

        let context = PolicyContext {
            client_id: Some(1),
            client_ip: Some("127.0.0.1".to_string()),
            operation: "CreateWindow".to_string(),
            resource_type: Some("window".to_string()),
            auth_method: Some("none".to_string()),
            permissions: vec![],
            resource_counts: std::iter::once(("window".to_string(), 500)).collect(),
            request_rate: 50.0,
            context: HashMap::new(),
        };

        let result = manager.evaluate_policies(&context);
        assert!(matches!(result.action, PolicyAction::Allow));
    }

    #[test]
    fn test_rate_limit_condition() {
        let mut manager = PolicyManager::new();
        let policy = PolicyManager::create_default_policy();
        manager.add_policy(policy).unwrap();

        let context = PolicyContext {
            client_id: Some(1),
            client_ip: Some("127.0.0.1".to_string()),
            operation: "CreateWindow".to_string(),
            resource_type: Some("window".to_string()),
            auth_method: Some("none".to_string()),
            permissions: vec![],
            resource_counts: HashMap::new(),
            request_rate: 150.0, // Exceeds rate limit
            context: HashMap::new(),
        };

        let result = manager.evaluate_policies(&context);
        assert!(matches!(result.action, PolicyAction::RateLimit { .. }));
        assert!(!result.matched_rules.is_empty());
    }

    #[test]
    fn test_resource_limit_condition() {
        let mut manager = PolicyManager::new();
        let policy = PolicyManager::create_default_policy();
        manager.add_policy(policy).unwrap();

        let context = PolicyContext {
            client_id: Some(1),
            client_ip: Some("127.0.0.1".to_string()),
            operation: "CreateWindow".to_string(),
            resource_type: Some("window".to_string()),
            auth_method: Some("none".to_string()),
            permissions: vec![],
            resource_counts: std::iter::once(("window".to_string(), 1500)).collect(), // Exceeds limit
            request_rate: 50.0,
            context: HashMap::new(),
        };

        let result = manager.evaluate_policies(&context);
        assert!(matches!(result.action, PolicyAction::Deny));
        assert!(!result.matched_rules.is_empty());
    }

    #[test]
    fn test_enable_disable_policy() {
        let mut manager = PolicyManager::new();
        let policy = PolicyManager::create_default_policy();
        manager.add_policy(policy).unwrap();

        assert!(manager.disable_policy("default").is_ok());
        assert!(!manager.get_policy("default").unwrap().active);

        assert!(manager.enable_policy("default").is_ok());
        assert!(manager.get_policy("default").unwrap().active);

        assert!(manager.enable_policy("nonexistent").is_err());
    }

    #[test]
    fn test_condition_evaluation() {
        let manager = PolicyManager::new();

        let context = PolicyContext {
            client_id: Some(42),
            client_ip: Some("192.168.1.100".to_string()),
            operation: "CreateWindow".to_string(),
            resource_type: Some("window".to_string()),
            auth_method: Some("password".to_string()),
            permissions: vec!["create_window".to_string()],
            resource_counts: std::iter::once(("window".to_string(), 10)).collect(),
            request_rate: 25.0,
            context: HashMap::new(),
        };

        // Test various conditions
        assert!(manager.evaluate_condition(&PolicyCondition::ClientId(42), &context));
        assert!(!manager.evaluate_condition(&PolicyCondition::ClientId(99), &context));

        assert!(manager.evaluate_condition(
            &PolicyCondition::Operation("CreateWindow".to_string()),
            &context
        ));
        assert!(!manager.evaluate_condition(
            &PolicyCondition::Operation("DestroyWindow".to_string()),
            &context
        ));

        assert!(manager.evaluate_condition(
            &PolicyCondition::HasPermission("create_window".to_string()),
            &context
        ));
        assert!(!manager.evaluate_condition(
            &PolicyCondition::HasPermission("admin".to_string()),
            &context
        ));

        assert!(manager.evaluate_condition(
            &PolicyCondition::ResourceCountExceeds {
                resource_type: "window".to_string(),
                count: 5
            },
            &context
        ));
        assert!(!manager.evaluate_condition(
            &PolicyCondition::ResourceCountExceeds {
                resource_type: "window".to_string(),
                count: 20
            },
            &context
        ));
    }
}
