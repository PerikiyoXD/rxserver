//! Task scheduler
//!
//! Provides scheduling capabilities for tasks and background operations.

use crate::server::runtime::{RuntimeError, RuntimeResult};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{Mutex, RwLock};
use tokio::time::{interval, sleep};

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Scheduled task definition
#[derive(Debug, Clone)]
pub struct ScheduledTask {
    /// Task ID
    pub id: String,
    /// Task name
    pub name: String,
    /// Task priority
    pub priority: TaskPriority,
    /// Next execution time
    pub next_run: SystemTime,
    /// Execution interval (for recurring tasks)
    pub interval: Option<Duration>,
    /// Maximum number of retries
    pub max_retries: u32,
    /// Current retry count
    pub retry_count: u32,
    /// Task enabled flag
    pub enabled: bool,
}

impl ScheduledTask {
    /// Create a new scheduled task
    pub fn new(id: String, name: String, next_run: SystemTime) -> Self {
        Self {
            id,
            name,
            priority: TaskPriority::Normal,
            next_run,
            interval: None,
            max_retries: 3,
            retry_count: 0,
            enabled: true,
        }
    }

    /// Set task priority
    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set recurring interval
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = Some(interval);
        self
    }

    /// Set maximum retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Check if task can be retried
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    /// Increment retry count
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }

    /// Reset retry count
    pub fn reset_retries(&mut self) {
        self.retry_count = 0;
    }

    /// Calculate next run time for recurring tasks
    pub fn calculate_next_run(&mut self) {
        if let Some(interval) = self.interval {
            self.next_run = SystemTime::now() + interval;
        }
    }
}

impl Eq for ScheduledTask {}

impl PartialEq for ScheduledTask {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Ord for ScheduledTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap behavior (earliest time first)
        other
            .next_run
            .cmp(&self.next_run)
            .then_with(|| other.priority.cmp(&self.priority))
    }
}

impl PartialOrd for ScheduledTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Task execution result
#[derive(Debug, Clone)]
pub enum TaskResult {
    Success,
    Failure(String),
    Retry(String),
}

/// Task scheduler
#[derive(Debug)]
pub struct TaskScheduler {
    /// Task queue (priority queue ordered by execution time)
    task_queue: Arc<Mutex<BinaryHeap<ScheduledTask>>>,
    /// Task registry for management
    tasks: Arc<RwLock<HashMap<String, ScheduledTask>>>,
    /// Running flag
    running: Arc<Mutex<bool>>,
}

impl TaskScheduler {
    /// Create a new task scheduler
    pub fn new() -> Self {
        Self {
            task_queue: Arc::new(Mutex::new(BinaryHeap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Initialize the task scheduler
    pub async fn initialize(&self) -> RuntimeResult<()> {
        // Task scheduler initialization logic
        let mut is_running = self.running.lock().await;
        *is_running = true;
        Ok(())
    }

    /// Start the scheduler
    pub async fn start(&self) -> RuntimeResult<()> {
        let mut running = self.running.lock().await;
        if *running {
            return Err(RuntimeError::Scheduler(
                "Scheduler already running".to_string(),
            ));
        }
        *running = true;

        // Spawn scheduler loop
        let task_queue = Arc::clone(&self.task_queue);
        let tasks = Arc::clone(&self.tasks);
        let running_flag = Arc::clone(&self.running);

        tokio::spawn(async move {
            let mut scheduler_interval = interval(Duration::from_millis(100));

            loop {
                scheduler_interval.tick().await;

                // Check if we should stop
                if !*running_flag.lock().await {
                    break;
                }

                // Process due tasks
                let now = SystemTime::now();
                let mut due_tasks = Vec::new();

                {
                    let mut queue = task_queue.lock().await;
                    while let Some(task) = queue.peek() {
                        if task.next_run <= now && task.enabled {
                            due_tasks.push(queue.pop().unwrap());
                        } else {
                            break;
                        }
                    }
                }

                // Execute due tasks
                for mut task in due_tasks {
                    // TODO: Execute the actual task logic here
                    let result = Self::execute_task(&task).await;

                    match result {
                        TaskResult::Success => {
                            task.reset_retries();
                            if task.interval.is_some() {
                                task.calculate_next_run();
                                task_queue.lock().await.push(task.clone());
                            }
                        }
                        TaskResult::Failure(_) | TaskResult::Retry(_) => {
                            if task.can_retry() {
                                task.increment_retry();
                                task.next_run = SystemTime::now() + Duration::from_secs(30); // Retry delay
                                task_queue.lock().await.push(task.clone());
                            }
                        }
                    }

                    // Update task registry
                    tasks.write().await.insert(task.id.clone(), task);
                }
            }
        });

        Ok(())
    }

    /// Stop the scheduler
    pub async fn stop(&self) -> RuntimeResult<()> {
        let mut running = self.running.lock().await;
        *running = false;
        Ok(())
    }

    /// Schedule a task
    pub async fn schedule_task(&self, task: ScheduledTask) -> RuntimeResult<()> {
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task.id.clone(), task.clone());
        }

        {
            let mut queue = self.task_queue.lock().await;
            queue.push(task);
        }

        Ok(())
    }

    /// Cancel a scheduled task
    pub async fn cancel_task(&self, task_id: &str) -> RuntimeResult<()> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.enabled = false;
            Ok(())
        } else {
            Err(RuntimeError::Scheduler(format!(
                "Task not found: {}",
                task_id
            )))
        }
    }

    /// Get task status
    pub async fn get_task(&self, task_id: &str) -> Option<ScheduledTask> {
        let tasks = self.tasks.read().await;
        tasks.get(task_id).cloned()
    }

    /// List all tasks
    pub async fn list_tasks(&self) -> Vec<ScheduledTask> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }

    /// Execute a task (placeholder implementation)
    async fn execute_task(_task: &ScheduledTask) -> TaskResult {
        // TODO: Implement actual task execution logic
        sleep(Duration::from_millis(10)).await;
        TaskResult::Success
    }

    /// Get scheduler statistics
    pub async fn get_stats(&self) -> SchedulerStats {
        let tasks = self.tasks.read().await;
        let queue = self.task_queue.lock().await;

        let total_tasks = tasks.len();
        let pending_tasks = queue.len();
        let enabled_tasks = tasks.values().filter(|t| t.enabled).count();
        let disabled_tasks = total_tasks - enabled_tasks;

        SchedulerStats {
            total_tasks,
            pending_tasks,
            enabled_tasks,
            disabled_tasks,
        }
    }
}

/// Scheduler statistics
#[derive(Debug, Clone)]
pub struct SchedulerStats {
    pub total_tasks: usize,
    pub pending_tasks: usize,
    pub enabled_tasks: usize,
    pub disabled_tasks: usize,
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}
