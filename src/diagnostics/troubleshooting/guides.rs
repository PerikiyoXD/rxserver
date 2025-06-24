use super::KnownIssue;

/// Manages a collection of troubleshooting guides.
#[derive(Debug)]
pub struct TroubleshootingGuideManager {
    guides: Vec<TroubleshootingGuide>,
}

impl TroubleshootingGuideManager {
    pub fn new() -> Self {
        Self { guides: Vec::new() }
    }

    pub fn add_guide(&mut self, guide: TroubleshootingGuide) {
        self.guides.push(guide);
    }

    pub fn find_guides_for_issue(&self, _issue: &KnownIssue) -> Vec<&TroubleshootingGuide> {
        todo!("Find relevant guides for the given issue")
    }
    pub fn get_guides_for_topic(&self, _topic: &str) -> Vec<&TroubleshootingGuide> {
        todo!("Get guides for specific topic")
    }

    pub fn get_all_guides(&self) -> &[TroubleshootingGuide] {
        &self.guides
    }
}

/// Represents a step-by-step troubleshooting guide.
#[derive(Debug, Clone)]
pub struct TroubleshootingGuide {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub steps: Vec<TroubleshootingStep>,
    pub related_issues: Vec<String>,
}

/// A single step in a troubleshooting guide.
#[derive(Debug, Clone)]
pub struct TroubleshootingStep {
    pub step_number: usize,
    pub title: String,
    pub description: String,
    pub commands: Vec<String>,
    pub expected_outcome: String,
}

impl TroubleshootingGuide {
    pub fn new(id: String, title: String, description: String, category: String) -> Self {
        Self {
            id,
            title,
            description,
            category,
            steps: Vec::new(),
            related_issues: Vec::new(),
        }
    }

    pub fn add_step(&mut self, step: TroubleshootingStep) {
        self.steps.push(step);
    }

    pub fn add_related_issue(&mut self, issue_id: String) {
        self.related_issues.push(issue_id);
    }
}

impl TroubleshootingStep {
    pub fn new(step_number: usize, title: String, description: String) -> Self {
        Self {
            step_number,
            title,
            description,
            commands: Vec::new(),
            expected_outcome: String::new(),
        }
    }

    pub fn add_command(&mut self, command: String) {
        self.commands.push(command);
    }

    pub fn set_expected_outcome(&mut self, outcome: String) {
        self.expected_outcome = outcome;
    }
}
