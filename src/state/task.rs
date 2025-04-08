//! Task state management.
//!
//! Manages the state of tasks running in the Crankshaft engine.

use chrono::{DateTime, Utc};

/// Task status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    Created,
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl TaskStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
        )
    }
    
    pub fn to_string(&self) -> &'static str {
        match self {
            TaskStatus::Created => "Created",
            TaskStatus::Queued => "Queued",
            TaskStatus::Running => "Running",
            TaskStatus::Completed => "Completed",
            TaskStatus::Failed => "Failed",
            TaskStatus::Cancelled => "Cancelled",
        }
    }
}

/// Task state.
#[derive(Debug, Clone)]
pub struct TaskState {
    pub id: u64,
    pub name: String,
    pub status: TaskStatus,
    pub progress: Option<f32>,
    pub backend: String,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub cancellation_token: Option<tokio_util::sync::CancellationToken>,
}

impl TaskState {
    pub fn new(
        id: u64,
        name: String,
        backend: String,
        cancellation_token: Option<tokio_util::sync::CancellationToken>,
    ) -> Self {
        Self {
            id,
            name,
            status: TaskStatus::Created,
            progress: None,
            backend,
            cpu_usage: 0.0,
            memory_usage: 0.0,
            start_time: Utc::now(),
            end_time: None,
            cancellation_token,
        }
    }
    
    pub fn duration(&self) -> chrono::Duration {
        self.end_time
            .unwrap_or_else(Utc::now)
            .signed_duration_since(self.start_time)
    }
    
    pub fn is_active(&self) -> bool {
        !self.status.is_terminal()
    }
    
    pub fn can_cancel(&self) -> bool {
        self.is_active() && self.cancellation_token.is_some()
    }
    
    pub fn elapsed(&self) -> chrono::Duration {
        if let Some(end_time) = self.end_time {
            end_time - self.start_time
        } else {
            chrono::Utc::now() - self.start_time
        }
    }
}
