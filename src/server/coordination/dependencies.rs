//! Dependency management
//!
//! Manages component dependencies and ensures proper initialization order.

use crate::server::coordination::CoordinationResult;
use std::collections::{HashMap, HashSet};

/// Component dependency manager
pub struct DependencyManager {
    dependencies: HashMap<String, Vec<String>>,
    resolved: HashSet<String>,
}

impl DependencyManager {
    /// Create a new dependency manager
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            resolved: HashSet::new(),
        }
    }

    /// Add a dependency relationship
    pub fn add_dependency(&mut self, component: String, depends_on: String) {
        self.dependencies
            .entry(component)
            .or_default()
            .push(depends_on);
    }

    /// Resolve dependencies and return initialization order
    pub fn resolve_order(&self) -> CoordinationResult<Vec<String>> {
        // TODO: Implement topological sort for dependency resolution
        Ok(vec![])
    }

    /// Check if a component's dependencies are satisfied
    pub fn dependencies_satisfied(&self, component: &str) -> bool {
        if let Some(deps) = self.dependencies.get(component) {
            deps.iter().all(|dep| self.resolved.contains(dep))
        } else {
            true // No dependencies
        }
    }

    /// Mark a component as resolved
    pub fn mark_resolved(&mut self, component: String) {
        self.resolved.insert(component);
    }
}

impl Default for DependencyManager {
    fn default() -> Self {
        Self::new()
    }
}
