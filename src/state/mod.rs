//! Application state management.
//!
//! This module implements the Entity-Component System pattern seen in tokio-console
//! for efficient state management.

mod task;
mod backend;
mod resource;

pub use task::{TaskState, TaskStatus};
pub use backend::{BackendState, HealthStatus, BackendKind};
pub use resource::ResourceState;

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use string_interner::{StringInterner, backend::SimpleBackend, DefaultSymbol};

/// Temporal state of the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Temporality {
    /// Live monitoring.
    Live,
    /// Paused monitoring.
    Paused,
    /// In the process of pausing.
    Pausing,
    /// In the process of unpausing.
    Unpausing,
}

/// Task details for the currently selected task.
#[derive(Debug, Clone)]
pub struct TaskDetails {
    /// ID of the task
    pub task_id: u64,
    /// Task logs
    pub logs: Vec<String>,
    /// Resource usage history
    pub resource_history: Vec<ResourceSample>,
    /// Start time of the task
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Elapsed time since task started
    pub elapsed: std::time::Duration,
}

/// Application state.
pub struct AppState {
    /// Task state container.
    pub tasks: HashMap<u64, TaskState>,
    /// Backend state container.
    pub backends: HashMap<String, BackendState>,
    /// Resource utilization state.
    pub resources: ResourceState,
    /// Currently selected task details.
    pub current_task_details: Option<Rc<RefCell<TaskDetails>>>,
    /// Monitoring state.
    pub temporality: Temporality,
    /// String interner for memory optimization.
    pub strings: StringInterner<SimpleBackend<DefaultSymbol>>,
    /// Last update time.
    pub last_update: std::time::Instant,
    /// Selected task ID (for UI state)
    pub selected_task_id: Option<u64>,
    /// Animation frame for UI updates.
    pub animation_frame: usize,
    /// Terminal width for UI layout.
    pub terminal_width: u16,
    /// Terminal height for UI layout.
    pub terminal_height: u16,
    /// Selected backend name (for UI state)
    pub selected_backend: Option<String>,
}

impl AppState {
    /// Creates a new application state.
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            backends: HashMap::new(),
            resources: ResourceState::new(),
            current_task_details: None,
            temporality: Temporality::Live,
            strings: StringInterner::new(),
            last_update: std::time::Instant::now(),
            selected_task_id: None,
            animation_frame: 0,
            terminal_width: 0,
            terminal_height: 0,
            selected_backend: None,
        }
    }
    
    /// Updates task states with new data.
    pub fn update_tasks(&mut self, updates: Vec<TaskUpdate>) {
        for update in updates {
            match update {
                TaskUpdate::Created(task) => {
                    // Intern strings to reduce memory usage
                    let _name = self.strings.get_or_intern(&task.name);
                    let _backend = self.strings.get_or_intern(&task.backend);
                    
                    self.tasks.insert(task.id, task);
                }
                TaskUpdate::StatusChanged(id, status) => {
                    if let Some(task) = self.tasks.get_mut(&id) {
                        task.status = status;
                    }
                }
                TaskUpdate::Progress(id, progress) => {
                    if let Some(task) = self.tasks.get_mut(&id) {
                        task.progress = Some(progress);
                    }
                }
                TaskUpdate::ResourceUsage(id, usage) => {
                    if let Some(task) = self.tasks.get_mut(&id) {
                        task.cpu_usage = usage.cpu;
                        task.memory_usage = usage.memory;
                        
                        // Update resource history if this is the selected task
                        if let Some(details) = &self.current_task_details {
                            if details.borrow().task_id == id {
                                details.borrow_mut().resource_history.push(ResourceSample {
                                    timestamp: chrono::Utc::now(),
                                    cpu: usage.cpu,
                                    memory: usage.memory,
                                });
                            }
                        }
                    }
                }
                TaskUpdate::Completed(id, result) => {
                    if let Some(task) = self.tasks.get_mut(&id) {
                        task.status = if result.is_ok() {
                            TaskStatus::Completed
                        } else {
                            TaskStatus::Failed
                        };
                        task.end_time = Some(chrono::Utc::now());
                    }
                }
                TaskUpdate::Logs(id, log) => {
                    // Add logs to task details if this is the selected task
                    if let Some(details) = &self.current_task_details {
                        if details.borrow().task_id == id {
                            details.borrow_mut().logs.push(log);
                        }
                    }
                }
            }
        }
        
        self.last_update = std::time::Instant::now();
    }
    
    /// Updates backend states with new data.
    pub fn update_backends(&mut self, updates: Vec<BackendUpdate>) {
        for update in updates {
            match update {
                BackendUpdate::Status(name, status) => {
                    let entry = self.backends.entry(name.clone()).or_insert_with(|| {
                        // Initialize a new backend state if needed
                        BackendState {
                            name,
                            kind: BackendKind::Unknown,
                            running_tasks: 0,
                            total_tasks: 0,
                            cpu_usage: 0.0,
                            memory_usage: 0.0,
                            health: HealthStatus::Unknown,
                            resource_history: Vec::new(),  // Add this field
                            last_update: chrono::Utc::now(),  // Add this field
                        }
                    });
                    
                    // Update backend state
                    entry.health = status.health;
                    entry.running_tasks = status.running_tasks;
                    entry.total_tasks = status.total_tasks;
                }
                BackendUpdate::ResourceUsage(name, usage) => {
                    if let Some(backend) = self.backends.get_mut(&name) {
                        backend.cpu_usage = usage.cpu;
                        backend.memory_usage = usage.memory;
                    }
                }
                BackendUpdate::Kind(name, kind) => {
                    if let Some(backend) = self.backends.get_mut(&name) {
                        backend.kind = kind;
                    }
                }
            }
        }
    }
    
    /// Selects a task for detailed view.
    pub fn select_task(&mut self, task_id: u64) {
        if let Some(task) = self.tasks.get(&task_id) {
            self.current_task_details = Some(Rc::new(RefCell::new(TaskDetails {
                task_id,
                logs: Vec::new(),
                resource_history: Vec::new(),
                start_time: task.start_time,
                elapsed: std::time::Duration::from_secs(0),
            })));
        }
    }
    
    /// Deselects the current task.
    pub fn deselect_task(&mut self) {
        self.current_task_details = None;
    }
    
    /// Get the currently selected task ID (if any)
    pub fn selected_task_id(&self) -> Option<&u64> {
        self.selected_task_id.as_ref()
    }
    
    /// Select the next task in the list
    pub fn select_next_task(&mut self) {
        if self.tasks.is_empty() {
            self.selected_task_id = None;
            return;
        }
        
        let current_id = self.selected_task_id;
        
        // Get all task IDs and sort them
        let mut task_ids: Vec<u64> = self.tasks.keys().copied().collect();
        task_ids.sort_unstable();
        
        // Find the next task ID
        if let Some(current_id) = current_id {
            if let Some(pos) = task_ids.iter().position(|&id| id == current_id) {
                if pos + 1 < task_ids.len() {
                    self.selected_task_id = Some(task_ids[pos + 1]);
                    return;
                }
            }
        }
        
        // If no current selection or current is last, select first
        if !task_ids.is_empty() {
            self.selected_task_id = Some(task_ids[0]);
        }
    }
    
    /// Select the previous task in the list
    pub fn select_prev_task(&mut self) {
        if self.tasks.is_empty() {
            self.selected_task_id = None;
            return;
        }
        
        let current_id = self.selected_task_id;
        
        // Get all task IDs and sort them
        let mut task_ids: Vec<u64> = self.tasks.keys().copied().collect();
        task_ids.sort_unstable();
        
        // Find the previous task ID
        if let Some(current_id) = current_id {
            if let Some(pos) = task_ids.iter().position(|&id| id == current_id) {
                if pos > 0 {
                    self.selected_task_id = Some(task_ids[pos - 1]);
                    return;
                }
            }
        }
        
        // If no current selection or current is first, select last
        if !task_ids.is_empty() {
            self.selected_task_id = Some(*task_ids.last().unwrap());
        }
    }
    
    // Similarly for backends
    pub fn selected_backend_name(&self) -> Option<String> {
        if self.backends.is_empty() {
            None
        } else {
            // For simplicity, we'll just return the first backend name
            // In a real app, you'd have a selected_backend_name field in AppState
            Some(self.backends.keys().next().unwrap().clone())
        }
    }
    
    pub fn select_next_backend(&mut self) {
        // Implementation similar to select_next_task, adapted for strings
    }
    
    pub fn select_prev_backend(&mut self) {
        // Implementation similar to select_prev_task, adapted for strings
    }
    
    /// Selects a backend for detailed view.
    pub fn select_backend(&mut self, name: &str) {
        self.selected_backend = Some(name.to_string());
    }
    
    /// Deselects the current backend.
    pub fn deselect_backend(&mut self) {
        self.selected_backend = None;
    }
    
    /// Returns the count of currently active tasks
    pub fn active_task_count(&self) -> usize {
        self.tasks
            .values()
            .filter(|task| {
                matches!(
                    task.status,
                    TaskStatus::Created | TaskStatus::Queued | TaskStatus::Running
                )
            })
            .count()
    }
    
    
}

/// Resource sample for historical tracking.
#[derive(Debug, Clone)]
pub struct ResourceSample {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub cpu: f32,
    pub memory: f32,
}

/// Task status update.
pub enum TaskUpdate {
    Created(TaskState),
    StatusChanged(u64, TaskStatus),
    Progress(u64, f32),
    ResourceUsage(u64, ResourceUsage),
    Completed(u64, Result<(), String>),
    Logs(u64, String),
}

/// Backend status update.
pub enum BackendUpdate {
    Status(String, BackendStatus),
    ResourceUsage(String, ResourceUsage),
    Kind(String, BackendKind),
}

/// Resource usage information.
#[derive(Debug, Clone, Copy)]
pub struct ResourceUsage {
    pub cpu: f32,
    pub memory: f32,
}

/// Backend status information.
#[derive(Debug, Clone)]
pub struct BackendStatus {
    pub running_tasks: usize,
    pub total_tasks: usize,
    pub health: HealthStatus,
}

/// Conversion from monitor TaskUpdate to state TaskUpdate
impl From<crate::monitor::task::TaskUpdate> for TaskUpdate {
    fn from(update: crate::monitor::task::TaskUpdate) -> Self {
        // First, check for log updates
        if let Some((id, message)) = update.logs {
            return TaskUpdate::Logs(id, message);
        }
        
        // Then check resource usage updates
        if let Some((id, sample)) = update.resource_usage {
            return TaskUpdate::ResourceUsage(id, ResourceUsage {
                cpu: sample.cpu,
                memory: sample.memory,
            });
        }
        
        // Check for new tasks
        if !update.new_tasks.is_empty() {
            let task_id = update.new_tasks[0];
            if let Some(task) = update.tasks.get(&task_id) {
                return TaskUpdate::Created(task.clone());
            }
        }
        
        // Check for completed tasks
        if !update.completed_tasks.is_empty() {
            let task_id = update.completed_tasks[0];
            return TaskUpdate::Completed(task_id, Ok(()));
        }
        
        // Finally, check for status updates
        if !update.updated_tasks.is_empty() {
            let task_id = update.updated_tasks[0];
            if let Some(task) = update.tasks.get(&task_id) {
                return TaskUpdate::StatusChanged(task_id, task.status);
            }
        }
        
        // Default fallback - create a dummy status update for the first task
        if let Some((&id, _)) = update.tasks.iter().next() {
            TaskUpdate::StatusChanged(id, TaskStatus::Running)
        } else {
            TaskUpdate::StatusChanged(0, TaskStatus::Created) // Empty update
        }
    }
}

/// Conversion from monitor BackendUpdate to state BackendUpdate 
impl From<crate::monitor::backend::BackendUpdate> for BackendUpdate {
    fn from(update: crate::monitor::backend::BackendUpdate) -> Self {
        // Take the first backend from the update
        if let Some((backend_name, backend_state)) = update.backends.iter().next() {
            // Create a status update for this backend
            return BackendUpdate::Status(
                backend_name.clone(), 
                BackendStatus {
                    running_tasks: backend_state.running_tasks,
                    total_tasks: backend_state.total_tasks,
                    health: backend_state.health,
                }
            );
        }
        
        // Empty update as fallback
        BackendUpdate::Status(
            "unknown".to_string(),
            BackendStatus {
                running_tasks: 0,
                total_tasks: 0,
                health: HealthStatus::Unknown,
            }
        )
    }
}