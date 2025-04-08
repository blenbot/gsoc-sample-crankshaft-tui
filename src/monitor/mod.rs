//! Monitoring components for gathering backend and task state information.
//!
//! This module provides functionality to connect to Crankshaft engines and 
//! monitor execution backends and tasks. These components handle:
//!
//! - Polling backends for health information
//! - Tracking task status and progress
//! - Converting engine-specific data into UI-friendly representations
//! - Resource usage tracking (CPU, memory, etc.)

pub mod backend;
pub mod task;

pub use backend::{BackendMonitor, BackendUpdate};
pub use task::{TaskMonitor, TaskUpdate};

use std::time::Duration;
use eyre::Result;

/// Default polling interval for backend status.
pub const DEFAULT_BACKEND_POLL_INTERVAL: Duration = Duration::from_secs(5);

/// Default polling interval for task status.
pub const DEFAULT_TASK_POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Monitor manager that handles connections to Crankshaft engines.
pub struct MonitorManager {
    /// Task monitor instance
    task_monitor: TaskMonitor,
    /// Backend monitor instance
    backend_monitor: BackendMonitor,
    /// Connection URL for the Crankshaft engine
    engine_url: String,
    /// Whether monitoring is active
    active: bool,
}

impl MonitorManager {
    /// Create a new monitor manager.
    pub fn new(engine_url: String) -> Self {
        Self {
            task_monitor: TaskMonitor::new(),
            backend_monitor: BackendMonitor::new(),
            engine_url,
            active: false,
        }
    }
    
    /// Connect to the Crankshaft engine.
    pub async fn connect(&mut self) -> Result<()> {
        // In a real implementation, this would establish a connection to the Crankshaft engine
        // For this sample project, we'll just set up the monitors with simulated data
        self.task_monitor.connect(&self.engine_url).await?;
        self.backend_monitor.connect(&self.engine_url).await?;
        
        self.active = true;
        Ok(())
    }
    
    /// Disconnect from the Crankshaft engine.
    pub async fn disconnect(&mut self) -> Result<()> {
        self.task_monitor.disconnect().await?;
        self.backend_monitor.disconnect().await?;
        
        self.active = false;
        Ok(())
    }
    
    /// Get the task monitor.
    pub fn task_monitor(&self) -> &TaskMonitor {
        &self.task_monitor
    }
    
    /// Get the backend monitor.
    pub fn backend_monitor(&self) -> &BackendMonitor {
        &self.backend_monitor
    }
    
    /// Check if monitoring is active.
    pub fn is_active(&self) -> bool {
        self.active
    }
}