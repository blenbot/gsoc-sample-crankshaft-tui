//! Backend state management.
//!
//! Manages the state of Crankshaft execution backends.

/// Backend type.
use chrono::{DateTime, Utc};
use crate::state::ResourceSample;

/// Health status of a backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendKind {
    Docker,
    TES,
    Generic,
    Local,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "Healthy"),
            HealthStatus::Degraded => write!(f, "Degraded"),
            HealthStatus::Unhealthy => write!(f, "Unhealthy"),
            HealthStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

impl std::fmt::Display for BackendKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendKind::Docker => write!(f, "Docker"),
            BackendKind::TES => write!(f, "TES"),
            BackendKind::Generic => write!(f, "Generic"),
            BackendKind::Local => write!(f, "Local"),
            BackendKind::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Backend state.
#[derive(Debug, Clone)]
pub struct BackendState {
    pub name: String,
    pub kind: BackendKind,
    pub running_tasks: usize,
    pub total_tasks: usize,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub health: HealthStatus,
    pub resource_history: Vec<ResourceSample>,
    pub last_update: DateTime<Utc>,
}

impl BackendState {
    pub fn new(name: String, kind: BackendKind) -> Self {
        Self {
            name,
            kind,
            running_tasks: 0,
            total_tasks: 0,
            cpu_usage: 0.0,
            memory_usage: 0.0,
            health: HealthStatus::Unknown,
            resource_history: Vec::new(),
            last_update: Utc::now(),
        }
    }
    
    pub fn utilization(&self) -> f32 {
        if self.total_tasks == 0 {
            0.0
        } else {
            self.running_tasks as f32 / self.total_tasks as f32
        }
    }
}