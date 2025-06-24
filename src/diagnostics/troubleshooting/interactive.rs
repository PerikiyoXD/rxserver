/// Interactive troubleshooting sessions.
///
/// This module provides an interactive interface for guided troubleshooting
/// where users can provide feedback and receive step-by-step assistance.
use crate::types::Result;

/// Interactive troubleshooting session.
#[derive(Debug)]
pub struct InteractiveTroubleshootingSession {
    session_id: String,
    issue_description: String,
    current_step: usize,
    steps: Vec<InteractiveStep>,
    user_responses: Vec<UserResponse>,
    state: SessionState,
}

/// State of an interactive session.
#[derive(Debug, Clone, Copy)]
pub enum SessionState {
    Starting,
    InProgress,
    WaitingForUser,
    Completed,
    Cancelled,
}

/// A single step in an interactive troubleshooting session.
#[derive(Debug, Clone)]
pub struct InteractiveStep {
    pub step_id: String,
    pub title: String,
    pub description: String,
    pub instruction: String,
    pub expected_responses: Vec<ExpectedResponse>,
    pub next_step_logic: NextStepLogic,
}

/// Response from the user.
#[derive(Debug, Clone)]
pub struct UserResponse {
    pub step_id: String,
    pub response_text: String,
    pub response_type: ResponseType,
    pub timestamp: std::time::SystemTime,
}

/// Expected response for a step.
#[derive(Debug, Clone)]
pub struct ExpectedResponse {
    pub pattern: String,
    pub next_step: Option<String>,
    pub interpretation: String,
}

/// Logic for determining the next step.
#[derive(Debug, Clone)]
pub enum NextStepLogic {
    Linear(String), // Next step ID
    Conditional(Vec<ConditionalNext>),
    End, // End of session
}

/// Conditional next step logic.
#[derive(Debug, Clone)]
pub struct ConditionalNext {
    pub condition: String, // Pattern to match
    pub next_step: String, // Next step ID if condition matches
}

/// Type of user response.
#[derive(Debug, Clone, Copy)]
pub enum ResponseType {
    YesNo,
    Text,
    Number,
    Choice,
}

/// Response to user interaction.
#[derive(Debug, Clone)]
pub struct InteractionResponse {
    pub message: String,
    pub next_instruction: Option<String>,
    pub possible_resolutions: Vec<String>,
    pub session_complete: bool,
}

impl InteractiveTroubleshootingSession {
    /// Creates a new interactive troubleshooting session.
    pub fn new(issue_description: String) -> Self {
        Self {
            session_id: format!(
                "session_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ),
            issue_description,
            current_step: 0,
            steps: Vec::new(),
            user_responses: Vec::new(),
            state: SessionState::Starting,
        }
    }

    pub fn get_session_id(&self) -> &str {
        &self.session_id
    }

    pub fn get_current_step(&self) -> Option<&InteractiveStep> {
        self.steps.get(self.current_step)
    }

    /// Processes user input and returns the next instruction.
    pub async fn process_user_input(&mut self, input: String) -> Result<InteractionResponse> {
        let response = UserResponse {
            step_id: self.current_step.to_string(),
            response_type: ResponseType::Text,
            response_text: input,
            timestamp: std::time::SystemTime::now(),
        };

        self.user_responses.push(response);

        // TODO: Process the input and determine next step
        Ok(InteractionResponse {
            message: "Continue with next step".to_string(),
            next_instruction: None,
            possible_resolutions: Vec::new(),
            session_complete: false,
        })
    }

    pub fn add_step(&mut self, step: InteractiveStep) {
        self.steps.push(step);
    }

    pub fn get_session_state(&self) -> SessionState {
        self.state
    }

    pub fn cancel_session(&mut self) {
        self.state = SessionState::Cancelled;
    }

    pub fn get_responses_history(&self) -> &[UserResponse] {
        &self.user_responses
    }
}

impl Clone for InteractiveTroubleshootingSession {
    fn clone(&self) -> Self {
        Self {
            session_id: self.session_id.clone(),
            issue_description: self.issue_description.clone(),
            current_step: self.current_step,
            steps: self.steps.clone(),
            user_responses: self.user_responses.clone(),
            state: self.state,
        }
    }
}

impl InteractiveStep {
    pub fn new(step_id: String, title: String, description: String, instruction: String) -> Self {
        Self {
            step_id,
            title,
            description,
            instruction,
            expected_responses: Vec::new(),
            next_step_logic: NextStepLogic::End,
        }
    }

    pub fn with_next_step(mut self, next_step_id: String) -> Self {
        self.next_step_logic = NextStepLogic::Linear(next_step_id);
        self
    }

    pub fn with_conditional_logic(mut self, conditions: Vec<ConditionalNext>) -> Self {
        self.next_step_logic = NextStepLogic::Conditional(conditions);
        self
    }

    pub fn add_expected_response(&mut self, response: ExpectedResponse) {
        self.expected_responses.push(response);
    }
}

impl UserResponse {
    pub fn new(step_id: String, response_text: String, response_type: ResponseType) -> Self {
        Self {
            step_id,
            response_text,
            response_type,
            timestamp: std::time::SystemTime::now(),
        }
    }
}

impl ExpectedResponse {
    pub fn new(pattern: String, interpretation: String) -> Self {
        Self {
            pattern,
            next_step: None,
            interpretation,
        }
    }

    pub fn with_next_step(mut self, next_step: String) -> Self {
        self.next_step = Some(next_step);
        self
    }
}

impl ConditionalNext {
    pub fn new(condition: String, next_step: String) -> Self {
        Self {
            condition,
            next_step,
        }
    }
}

impl InteractionResponse {
    pub fn new(message: String) -> Self {
        Self {
            message,
            next_instruction: None,
            possible_resolutions: Vec::new(),
            session_complete: false,
        }
    }

    pub fn with_next_instruction(mut self, instruction: String) -> Self {
        self.next_instruction = Some(instruction);
        self
    }

    pub fn with_resolutions(mut self, resolutions: Vec<String>) -> Self {
        self.possible_resolutions = resolutions;
        self
    }

    pub fn set_complete(mut self) -> Self {
        self.session_complete = true;
        self
    }
}
