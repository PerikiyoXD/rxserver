//! Resource dependency tracking and management
//!
//! This module handles dependencies between resources, ensuring proper cleanup
//! order and preventing resource leaks.

use crate::x11::protocol::types::XId;
use std::collections::{HashMap, HashSet};

/// Errors that can occur during dependency operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyError {
    /// Circular dependency detected
    CircularDependency(Vec<XId>),
    /// Dependency not found
    DependencyNotFound(XId),
    /// Resource has unresolved dependencies
    UnresolvedDependencies(XId, Vec<XId>),
}

impl std::fmt::Display for DependencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyError::CircularDependency(cycle) => {
                write!(f, "Circular dependency detected: {:?}", cycle)
            }
            DependencyError::DependencyNotFound(xid) => {
                write!(f, "Dependency {} not found", xid)
            }
            DependencyError::UnresolvedDependencies(xid, deps) => {
                write!(
                    f,
                    "Resource {} has unresolved dependencies: {:?}",
                    xid, deps
                )
            }
        }
    }
}

impl std::error::Error for DependencyError {}

/// Tracks dependencies between resources
#[derive(Debug, Default)]
pub struct DependencyTracker {
    /// Maps each resource to its dependencies (resources it depends on)
    dependencies: HashMap<XId, HashSet<XId>>,
    /// Maps each resource to its dependents (resources that depend on it)
    dependents: HashMap<XId, HashSet<XId>>,
}

impl DependencyTracker {
    /// Create a new dependency tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a dependency relationship: dependent depends on dependency
    pub fn add_dependency(
        &mut self,
        dependent: XId,
        dependency: XId,
    ) -> Result<(), DependencyError> {
        // Check for circular dependencies before adding
        if self.would_create_cycle(dependent, dependency) {
            let cycle = self.find_cycle(dependent, dependency);
            return Err(DependencyError::CircularDependency(cycle));
        }

        // Add the dependency
        self.dependencies
            .entry(dependent)
            .or_insert_with(HashSet::new)
            .insert(dependency);

        // Add the reverse mapping
        self.dependents
            .entry(dependency)
            .or_insert_with(HashSet::new)
            .insert(dependent);

        Ok(())
    }

    /// Remove a dependency relationship
    pub fn remove_dependency(&mut self, dependent: XId, dependency: XId) {
        if let Some(deps) = self.dependencies.get_mut(&dependent) {
            deps.remove(&dependency);
            if deps.is_empty() {
                self.dependencies.remove(&dependent);
            }
        }

        if let Some(deps) = self.dependents.get_mut(&dependency) {
            deps.remove(&dependent);
            if deps.is_empty() {
                self.dependents.remove(&dependency);
            }
        }
    }

    /// Remove all dependencies for a resource (when it's being destroyed)
    pub fn remove_resource(&mut self, xid: XId) {
        // Remove as a dependent from all dependencies
        if let Some(dependencies) = self.dependencies.remove(&xid) {
            for dependency in dependencies {
                if let Some(deps) = self.dependents.get_mut(&dependency) {
                    deps.remove(&xid);
                    if deps.is_empty() {
                        self.dependents.remove(&dependency);
                    }
                }
            }
        }

        // Remove as a dependency from all dependents
        if let Some(dependents) = self.dependents.remove(&xid) {
            for dependent in dependents {
                if let Some(deps) = self.dependencies.get_mut(&dependent) {
                    deps.remove(&xid);
                    if deps.is_empty() {
                        self.dependencies.remove(&dependent);
                    }
                }
            }
        }
    }

    /// Get direct dependencies of a resource
    pub fn get_dependencies(&self, xid: XId) -> Vec<XId> {
        self.dependencies
            .get(&xid)
            .map(|deps| deps.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Get direct dependents of a resource
    pub fn get_dependents(&self, xid: XId) -> Vec<XId> {
        self.dependents
            .get(&xid)
            .map(|deps| deps.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Get all transitive dependencies (dependencies of dependencies)
    pub fn get_all_dependencies(&self, xid: XId) -> Vec<XId> {
        let mut visited = HashSet::new();
        let mut result = Vec::new();
        self.collect_dependencies(xid, &mut visited, &mut result);
        result
    }

    /// Get all transitive dependents (dependents of dependents)
    pub fn get_all_dependents(&self, xid: XId) -> Vec<XId> {
        let mut visited = HashSet::new();
        let mut result = Vec::new();
        self.collect_dependents(xid, &mut visited, &mut result);
        result
    }

    /// Check if removing a resource would leave any unresolved dependencies
    pub fn check_removal_safety(&self, xid: XId) -> Result<(), DependencyError> {
        let dependents = self.get_dependents(xid);
        if !dependents.is_empty() {
            return Err(DependencyError::UnresolvedDependencies(xid, dependents));
        }
        Ok(())
    }

    /// Get a safe destruction order for a set of resources
    pub fn destruction_order(&self, resources: &[XId]) -> Result<Vec<XId>, DependencyError> {
        let mut order = Vec::new();
        let mut remaining: HashSet<XId> = resources.iter().copied().collect();

        while !remaining.is_empty() {
            let mut progress = false;

            // Find resources with no dependencies in the remaining set
            for &xid in &remaining.clone() {
                let deps = self.get_dependencies(xid);
                let has_remaining_deps = deps.iter().any(|dep| remaining.contains(dep));

                if !has_remaining_deps {
                    order.push(xid);
                    remaining.remove(&xid);
                    progress = true;
                }
            }

            if !progress {
                // Circular dependency or impossible situation
                let cycle: Vec<XId> = remaining.into_iter().collect();
                return Err(DependencyError::CircularDependency(cycle));
            }
        }

        Ok(order)
    }

    /// Check if adding a dependency would create a cycle
    fn would_create_cycle(&self, dependent: XId, dependency: XId) -> bool {
        if dependent == dependency {
            return true;
        }

        let mut visited = HashSet::new();
        self.has_path(dependency, dependent, &mut visited)
    }

    /// Find a cycle that would be created
    fn find_cycle(&self, dependent: XId, dependency: XId) -> Vec<XId> {
        let mut path = Vec::new();
        let mut visited = HashSet::new();

        if self.find_path(dependency, dependent, &mut visited, &mut path) {
            path.push(dependency);
            path
        } else {
            vec![dependent, dependency]
        }
    }

    /// Check if there's a path from source to target
    fn has_path(&self, source: XId, target: XId, visited: &mut HashSet<XId>) -> bool {
        if source == target {
            return true;
        }

        if visited.contains(&source) {
            return false;
        }

        visited.insert(source);

        if let Some(deps) = self.dependencies.get(&source) {
            for &dep in deps {
                if self.has_path(dep, target, visited) {
                    return true;
                }
            }
        }

        false
    }

    /// Find a path from source to target
    fn find_path(
        &self,
        source: XId,
        target: XId,
        visited: &mut HashSet<XId>,
        path: &mut Vec<XId>,
    ) -> bool {
        if source == target {
            return true;
        }

        if visited.contains(&source) {
            return false;
        }

        visited.insert(source);
        path.push(source);

        if let Some(deps) = self.dependencies.get(&source) {
            for &dep in deps {
                if self.find_path(dep, target, visited, path) {
                    return true;
                }
            }
        }

        path.pop();
        false
    }

    /// Recursively collect all dependencies
    fn collect_dependencies(&self, xid: XId, visited: &mut HashSet<XId>, result: &mut Vec<XId>) {
        if visited.contains(&xid) {
            return;
        }

        visited.insert(xid);

        if let Some(deps) = self.dependencies.get(&xid) {
            for &dep in deps {
                result.push(dep);
                self.collect_dependencies(dep, visited, result);
            }
        }
    }

    /// Recursively collect all dependents
    fn collect_dependents(&self, xid: XId, visited: &mut HashSet<XId>, result: &mut Vec<XId>) {
        if visited.contains(&xid) {
            return;
        }

        visited.insert(xid);

        if let Some(deps) = self.dependents.get(&xid) {
            for &dep in deps {
                result.push(dep);
                self.collect_dependents(dep, visited, result);
            }
        }
    }
}
