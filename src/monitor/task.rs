//! Task monitoring functionality.
//!
//! This module handles connecting to and monitoring Crankshaft tasks,
//! tracking their status, progress, and resource usage.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{self, Duration};
use eyre::Result;
use rand::{Rng, thread_rng, seq::SliceRandom};
use rand::rngs::StdRng;
use rand::SeedableRng;
use chrono::{DateTime, Utc, Duration as ChronoDuration};

use crate::state::{TaskState, TaskStatus, ResourceSample};
use super::DEFAULT_TASK_POLL_INTERVAL;

/// Update containing task state information.
#[derive(Debug, Clone)]
pub struct TaskUpdate {
    /// Map of task ID to state
    pub tasks: HashMap<u64, TaskState>,
    /// Timestamp of update
    pub timestamp: DateTime<Utc>,
    /// IDs of new tasks
    pub new_tasks: Vec<u64>,
    /// IDs of updated tasks
    pub updated_tasks: Vec<u64>,
    /// IDs of completed tasks
    pub completed_tasks: Vec<u64>,
    /// Resource usage update
    pub resource_usage: Option<(u64, ResourceSample)>,
    /// Log message update
    pub logs: Option<(u64, String)>,
}

/// Task monitor for tracking execution task status.
pub struct TaskMonitor {
    /// Sender for task updates
    update_sender: Option<mpsc::Sender<TaskUpdate>>,
    /// Receiver for task updates
    update_receiver: Option<mpsc::Receiver<TaskUpdate>>,
    /// Polling interval
    poll_interval: Duration,
    /// Connection URL
    connection_url: Option<String>,
    /// Demo mode flag (for generating sample data)
    demo_mode: bool,
    /// In-memory task states (for demo mode)
    task_states: Arc<Mutex<HashMap<u64, TaskState>>>,
    /// Next task ID to assign (for demo mode)
    next_task_id: Arc<Mutex<u64>>,
}

impl TaskMonitor {
    /// Create a new task monitor.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);
        
        Self {
            update_sender: Some(tx),
            update_receiver: Some(rx),
            poll_interval: DEFAULT_TASK_POLL_INTERVAL,
            connection_url: None,
            demo_mode: true,
            task_states: Arc::new(Mutex::new(HashMap::new())),
            next_task_id: Arc::new(Mutex::new(1)),
        }
    }
    
    /// Connect to the monitoring endpoint.
    pub async fn connect(&mut self, url: &str) -> Result<()> {
        self.connection_url = Some(url.to_string());
        
        if self.demo_mode {
            // Initialize demo tasks
            let mut tasks = HashMap::new();
            let now = Utc::now();
            
            // Create initial tasks
            self.add_demo_task(
                &mut tasks, 
                1, 
                "genome-analysis".to_string(), 
                "docker-local".to_string(), 
                TaskStatus::Running,
                Some(0.75), 
                now - ChronoDuration::minutes(15),
                None
            );
            
            self.add_demo_task(
                &mut tasks, 
                2, 
                "data-preprocessing".to_string(), 
                "local-runner".to_string(), 
                TaskStatus::Completed,
                Some(1.0), 
                now - ChronoDuration::hours(1),
                Some(now - ChronoDuration::minutes(10))
            );
            
            self.add_demo_task(
                &mut tasks, 
                3, 
                "batch-processing".to_string(), 
                "tes-cloud".to_string(), 
                TaskStatus::Running,
                Some(0.35), 
                now - ChronoDuration::minutes(45),
                None 
            );
            
            self.add_demo_task(
                &mut tasks, 
                4, 
                "alignment-job".to_string(), 
                "tes-cloud".to_string(), 
                TaskStatus::Running,
                Some(0.15), 
                now - ChronoDuration::minutes(5),
                None 
            );
            
            self.add_demo_task(
                &mut tasks, 
                5, 
                "failed-workflow".to_string(), 
                "docker-local".to_string(), 
                TaskStatus::Failed,
                Some(0.6), 
                now - ChronoDuration::hours(2),
                Some(now - ChronoDuration::hours(1))
            );
            
            // Store the tasks and set the next task ID
            {
                let mut state = self.task_states.lock().await;
                *state = tasks;
                
                let mut next_id = self.next_task_id.lock().await;
                *next_id = 6;
            }
            
            // Start the demo polling task
            self.start_demo_polling().await?;
        } else {
            // In a real implementation, this would connect to a real Crankshaft engine
            // and start polling for task status
            // self.start_real_polling().await?;
        }
        
        Ok(())
    }
    
    /// Add a demo task to the tasks map.
    fn add_demo_task(
        &self,
        tasks: &mut HashMap<u64, TaskState>,
        id: u64,
        name: String,
        backend: String,
        status: TaskStatus,
        progress: Option<f32>,
        start_time: DateTime<Utc>,
        end_time: Option<DateTime<Utc>>, 
    ) {
        // Generate some resource samples
        let mut rng = thread_rng();
        let sample_count = rng.gen_range(10..30);
        let mut resource_samples = Vec::with_capacity(sample_count);
        
        let cpu_base = rng.gen_range(10.0_f32..50.0_f32);
        let mem_base = rng.gen_range(50.0_f32..200.0_f32);
        
        for i in 0..sample_count {
            let timestamp = start_time + ChronoDuration::seconds(i as i64 * 10);
            
            // If the task has ended, don't generate samples past the end time
            if let Some(end) = end_time {
                if timestamp > end {
                    break;
                }
            }
            
            let cpu_jitter = rng.gen_range(-5.0_f32..5.0_f32);
            let mem_jitter = rng.gen_range(-10.0_f32..10.0_f32);
            
            resource_samples.push(ResourceSample {
                timestamp,
                cpu: (cpu_base + cpu_jitter).max(0.0_f32),    
                memory: (mem_base + mem_jitter).max(0.0_f32), 
            });
        }
        
        // Create the task
        let task = TaskState {
            id,
            name,
            backend,
            status,
            progress,
            cpu_usage: resource_samples.last().map_or(0.0, |s| s.cpu),
            memory_usage: resource_samples.last().map_or(0.0, |s| s.memory),
            start_time,
            end_time,
            cancellation_token: None,
        };
        
        tasks.insert(id, task);
    }
    
    /// Start the demo polling task.
    async fn start_demo_polling(&self) -> Result<()> {
        // Clone the necessary data for the polling task
        let task_states = Arc::clone(&self.task_states);
        let next_task_id = Arc::clone(&self.next_task_id);
        let sender = self.update_sender.as_ref().unwrap().clone();
        let interval = self.poll_interval;
        
        tokio::spawn(async move {
            let mut rng = StdRng::from_entropy();
            let mut interval_timer = time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                let mut states = task_states.lock().await;
                
                // Create a snapshot of states for use in updates
                let states_snapshot = states.clone();
                
                let mut new_tasks = Vec::new();
                let mut updated_tasks = Vec::new();
                let mut completed_tasks = Vec::new();
                
                // Generate random new task
                if rng.gen_ratio(1, 30) {
                    let mut next_id = next_task_id.lock().await;
                    let id = *next_id;
                    *next_id += 1;
                    
                    let task_names = [
                        "sequence-analysis", "data-processing", "variant-calling", 
                        "alignment-job", "fastq-conversion", "sam-to-bam", 
                        "quality-control", "trim-adapters", "demultiplexing"
                    ];
                    
                    let backend_names = ["docker-local", "tes-cloud", "local-runner"];
                    
                    let name = format!("{}-{}", task_names.choose(&mut rng).unwrap(), id);
                    let backend = backend_names.choose(&mut rng).unwrap().to_string();
                    
                    // Use the same pattern in add_demo_task_static
                    Self::add_demo_task_static(
                        &mut states, id, name, backend, TaskStatus::Created,
                        None, Utc::now(), None
                    );
                    
                    new_tasks.push(id);
                }
                
                // Process existing tasks
                for (id, task) in states.iter_mut() {
                    // New scope with a fresh RNG for each task
                    let updated = {
                        let mut updated = false;
                        
                        // Update the task using this RNG
                        if task.status == TaskStatus::Created || task.status == TaskStatus::Queued || task.status == TaskStatus::Running {
                            // Process status transitions
                            if task.status == TaskStatus::Created && rng.gen_ratio(1, 3) {
                                task.status = TaskStatus::Queued;
                                updated = true;
                            } else if task.status == TaskStatus::Queued && rng.gen_ratio(1, 3) {
                                task.status = TaskStatus::Running;
                                updated = true;
                            }
                            
                            if task.status == TaskStatus::Running {
                                // Update progress
                                if let Some(progress) = &mut task.progress {
                                    *progress += rng.gen_range(0.01..0.05);
                                    updated = true;
                                    
                                    // Complete or fail the task
                                    if *progress >= 1.0 {
                                        *progress = 1.0;
                                        task.status = if rng.gen_ratio(8, 10) {
                                            TaskStatus::Completed
                                        } else {
                                            TaskStatus::Failed
                                        };
                                        task.end_time = Some(Utc::now());
                                        completed_tasks.push(*id);
                                    }
                                } else {
                                    task.progress = Some(0.0);
                                    updated = true;
                                }
                                
                                // Update resource usage
                                let cpu_delta = rng.gen_range(-3.0..3.0);
                                task.cpu_usage = (task.cpu_usage + cpu_delta).max(0.0_f32);
                                
                                let mem_delta = rng.gen_range(-5.0..5.0);
                                task.memory_usage = (task.memory_usage + mem_delta).max(0.0_f32);
                            }
                        }
                        
                        updated
                    };
                    
                    // Send resource samples - using states_snapshot instead
                    if task.status == TaskStatus::Running {
                        let resource_sample = ResourceSample {
                            timestamp: Utc::now(),
                            cpu: task.cpu_usage,
                            memory: task.memory_usage,
                        };
                        
                        if sender.send(TaskUpdate {
                            tasks: states_snapshot.clone(), // Use snapshot instead of states
                            timestamp: Utc::now(),
                            new_tasks: Vec::new(),
                            updated_tasks: Vec::new(),
                            completed_tasks: Vec::new(),
                            resource_usage: Some((*id, resource_sample)),
                            logs: None,
                        }).await.is_err() {
                            return;
                        }
                        
                        // Generate log message (new RNG after await)
                        if rng.gen_ratio(1, 10) {
                            let log_messages = [
                                "Processing input file...",
                                "Loading reference genome...",
                                "Running alignment step...",
                                "Calculating statistics...",
                                "Optimizing parameters...",
                                "Writing output file...",
                                "Validating results...",
                                "WARNING: Low disk space",
                                "INFO: Checkpoint saved",
                                "DEBUG: Memory usage stable",
                            ];
                            
                            let log_message = format!("[{}] {}", 
                                Utc::now().format("%H:%M:%S"),
                                log_messages.choose(&mut rng).unwrap()
                            );
                            
                            // Send the log update
                            if sender.send(TaskUpdate {
                                tasks: states_snapshot.clone(), // Use snapshot here too
                                timestamp: Utc::now(),
                                new_tasks: Vec::new(),
                                updated_tasks: Vec::new(),
                                completed_tasks: Vec::new(),
                                resource_usage: None,
                                logs: Some((*id, log_message)),
                            }).await.is_err() {
                                return;
                            }
                        }
                    }
                    
                    if updated {
                        updated_tasks.push(*id);
                    }
                }
                
                // Send the combined update
                let update = TaskUpdate {
                    tasks: states_snapshot.clone(), // Use snapshot here too
                    timestamp: Utc::now(),
                    new_tasks,
                    updated_tasks,
                    completed_tasks,
                    resource_usage: None,
                    logs: None,
                };
                
                if sender.send(update).await.is_err() {
                    break;
                }
            }
        });
        
        Ok(())
    }
    
    /// Static helper to add a demo task (used by the polling task).
    fn add_demo_task_static(
        tasks: &mut HashMap<u64, TaskState>,
        id: u64,
        name: String,
        backend: String,
        status: TaskStatus,
        progress: Option<f32>,
        start_time: DateTime<Utc>,
        end_time: Option<DateTime<Utc>>, 
    ) {
        // Generate some resource samples
        let mut rng = StdRng::from_entropy();
        let sample_count = rng.gen_range(5..15);
        let mut resource_samples = Vec::with_capacity(sample_count);
        
        let cpu_base = rng.gen_range(10.0_f32..50.0_f32);
        let mem_base = rng.gen_range(50.0_f32..200.0_f32);
        
        for i in 0..sample_count {
            let timestamp = start_time + ChronoDuration::seconds(i as i64 * 10);
            
            // If the task has ended, don't generate samples past the end time
            if let Some(end) = end_time {
                if timestamp > end {
                    break;
                }
            }
            
            let cpu_jitter = rng.gen_range(-5.0_f32..5.0_f32);
            let mem_jitter = rng.gen_range(-10.0_f32..10.0_f32);
            
            resource_samples.push(ResourceSample {
                timestamp,
                cpu: (cpu_base + cpu_jitter).max(0.0_f32),    
                memory: (mem_base + mem_jitter).max(0.0_f32), 
            });
        }
        
        // Create the task
        let task = TaskState {
            id,
            name,
            backend,
            status,
            progress,
            cpu_usage: resource_samples.last().map_or(0.0, |s| s.cpu),
            memory_usage: resource_samples.last().map_or(0.0, |s| s.memory),
            start_time,
            end_time,
            cancellation_token: None, 
        };
        
        tasks.insert(id, task);
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
    pub async fn poll(&mut self) -> Option<TaskUpdate> {
        if let Some(receiver) = &mut self.update_receiver {
            receiver.try_recv().ok()
        } else {
            None
        }
    }
}

impl Default for TaskMonitor {
    fn default() -> Self {
        Self::new()
    }
}