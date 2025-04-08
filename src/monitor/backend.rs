//! Backend monitoring functionality.
//!
//! This module handles connecting to and monitoring Crankshaft execution backends,
//! tracking their health status and resource usage.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{self, Duration};
use eyre::Result;
use rand::{Rng, rngs::StdRng, SeedableRng};
use chrono::{DateTime, Utc};

use crate::state::{BackendState, HealthStatus, BackendKind, ResourceSample};
use super::DEFAULT_BACKEND_POLL_INTERVAL;

/// Update containing backend state information.
#[derive(Debug, Clone)]
pub struct BackendUpdate {
    /// Map of backend name to state
    pub backends: HashMap<String, BackendState>,
    /// Timestamp of update
    pub timestamp: DateTime<Utc>,
}

/// Backend monitor for tracking execution backend health.
pub struct BackendMonitor {
    /// Sender for backend updates
    update_sender: Option<mpsc::Sender<BackendUpdate>>,
    /// Receiver for backend updates
    update_receiver: Option<mpsc::Receiver<BackendUpdate>>,
    /// Polling interval
    poll_interval: Duration,
    /// Connection URL
    connection_url: Option<String>,
    /// Demo mode flag (for generating sample data)
    demo_mode: bool,
    /// In-memory backend states (for demo mode)
    backend_states: Arc<Mutex<HashMap<String, BackendState>>>,
}

impl BackendMonitor {
    /// Create a new backend monitor.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);
        
        Self {
            update_sender: Some(tx),
            update_receiver: Some(rx),
            poll_interval: DEFAULT_BACKEND_POLL_INTERVAL,
            connection_url: None,
            demo_mode: true,
            backend_states: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Connect to the monitoring endpoint.
    pub async fn connect(&mut self, url: &str) -> Result<()> {
        self.connection_url = Some(url.to_string());
        
        if self.demo_mode {
            // Initialize demo backends
            let mut backends = HashMap::new();
            
            // Add Docker backend
            backends.insert("docker-local".to_string(), BackendState {
                name: "docker-local".to_string(),
                kind: BackendKind::Docker,
                health: HealthStatus::Healthy,
                running_tasks: 3,
                total_tasks: 5,
                cpu_usage: 45.2,
                memory_usage: 32.8,
                resource_history: Vec::new(),
                last_update: Utc::now(),
            });
            
            // Add TES backend
            backends.insert("tes-cloud".to_string(), BackendState {
                name: "tes-cloud".to_string(),
                kind: BackendKind::TES,
                health: HealthStatus::Degraded,
                running_tasks: 12,
                total_tasks: 30,
                cpu_usage: 78.5,
                memory_usage: 65.3,
                resource_history: Vec::new(),
                last_update: Utc::now(),
            });
            
            // Add Generic backend
            backends.insert("local-runner".to_string(), BackendState {
                name: "local-runner".to_string(),
                kind: BackendKind::Generic,
                health: HealthStatus::Healthy,
                running_tasks: 1,
                total_tasks: 2,
                cpu_usage: 12.3,
                memory_usage: 8.7,
                resource_history: Vec::new(),
                last_update: Utc::now(),
            });
            
            // Store the backends
            {
                let mut state = self.backend_states.lock().await;
                *state = backends;
            }
            
            // Start the demo polling task
            self.start_demo_polling().await?;
        } else {
            // In a real implementation, this would connect to a real Crankshaft engine
            // and start polling for backend status
            // self.start_real_polling().await?;
        }
        
        Ok(())
    }
    
    /// Start the demo polling task.
    async fn start_demo_polling(&self) -> Result<()> {
        // Clone the necessary data for the polling task
        let backend_states = Arc::clone(&self.backend_states);
        let sender = self.update_sender.as_ref().unwrap().clone();
        let interval = self.poll_interval;
        
        tokio::spawn(async move {
            // Create a thread-safe RNG instead of thread_rng()
            let mut rng = StdRng::from_entropy();
            let mut interval_timer = time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                // Update backend states with simulated changes
                // No need for thread_rng() here anymore
                let mut states = backend_states.lock().await;
                
                // Update each backend using our thread-safe rng
                for (_, backend) in states.iter_mut() {
                    // Randomly adjust CPU usage
                    let cpu_delta = rng.gen_range(-5.0..5.0);
                    backend.cpu_usage = (backend.cpu_usage + cpu_delta).clamp(0.0, 100.0);
                    
                    // Randomly adjust memory usage
                    let mem_delta = rng.gen_range(-3.0..3.0);
                    backend.memory_usage = (backend.memory_usage + mem_delta).clamp(0.0, 100.0);
                    
                    // Occasionally change health status for the TES backend (to simulate issues)
                    if backend.kind == BackendKind::TES && rng.gen_ratio(1, 20) {
                        let statuses = [HealthStatus::Healthy, HealthStatus::Degraded, HealthStatus::Unhealthy];
                        backend.health = statuses[rng.gen_range(0..3)];
                    }
                    
                    // Update the timestamp
                    backend.last_update = Utc::now();
                    
                    // Add resource sample
                    backend.resource_history.push(ResourceSample {
                        timestamp: Utc::now(),
                        cpu: backend.cpu_usage,
                        memory: backend.memory_usage,
                    });
                    
                    // Keep only the last 60 samples (5 minutes at 5s interval)
                    if backend.resource_history.len() > 60 {
                        backend.resource_history.remove(0);
                    }
                }
                
                // Create the update
                let update = BackendUpdate {
                    backends: states.clone(),
                    timestamp: Utc::now(),
                };
                
                // Send the update
                if sender.send(update).await.is_err() {
                    // Channel closed, exit the task
                    break;
                }
            }
        });
        
        Ok(())
    }
    
    /// Disconnect from the monitoring endpoint.
    pub async fn disconnect(&mut self) -> Result<()> {
        self.connection_url = None;
        Ok(())
    }
    
    /// Set the polling interval.
    pub fn set_poll_interval(&mut self, interval: Duration) {
        self.poll_interval = interval;
    }
    
    /// Poll for updates.
    pub async fn poll(&mut self) -> Option<BackendUpdate> {
        if let Some(receiver) = &mut self.update_receiver {
            receiver.try_recv().ok()
        } else {
            None
        }
    }
}

impl Default for BackendMonitor {
    fn default() -> Self {
        Self::new()
    }
}



