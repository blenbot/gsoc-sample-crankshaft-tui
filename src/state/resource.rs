//! Resource state management.
//!
//! Tracks resource utilization across tasks and backends.

use std::collections::VecDeque;
use chrono::{DateTime, Utc};

const HISTORY_SIZE: usize = 100; // Keep 100 samples max

/// Resource utilization state.
pub struct ResourceState {
    pub cpu_history: VecDeque<ResourcePoint>,
    pub memory_history: VecDeque<ResourcePoint>,
    pub cpu_current: f32,
    pub memory_current: f32,
}

/// A single resource utilization data point.
#[derive(Debug, Clone, Copy)]
pub struct ResourcePoint {
    pub timestamp: DateTime<Utc>,
    pub value: f32,
}

impl ResourceState {
    pub fn new() -> Self {
        Self {
            cpu_history: VecDeque::with_capacity(HISTORY_SIZE),
            memory_history: VecDeque::with_capacity(HISTORY_SIZE),
            cpu_current: 0.0,
            memory_current: 0.0,
        }
    }
    
    pub fn update(&mut self, cpu: f32, memory: f32) {
        let now = Utc::now();
        
        // Update current values
        self.cpu_current = cpu;
        self.memory_current = memory;
        
        // Add to history
        self.add_cpu_point(now, cpu);
        self.add_memory_point(now, memory);
    }
    
    fn add_cpu_point(&mut self, timestamp: DateTime<Utc>, value: f32) {
        self.cpu_history.push_back(ResourcePoint { timestamp, value });
        if self.cpu_history.len() > HISTORY_SIZE {
            self.cpu_history.pop_front();
        }
    }
    
    fn add_memory_point(&mut self, timestamp: DateTime<Utc>, value: f32) {
        self.memory_history.push_back(ResourcePoint { timestamp, value });
        if self.memory_history.len() > HISTORY_SIZE {
            self.memory_history.pop_front();
        }
    }
    
    pub fn cpu_max(&self) -> f32 {
        self.cpu_history
            .iter()
            .map(|p| p.value)
            .fold(0.0, f32::max)
    }
    
    pub fn memory_max(&self) -> f32 {
        self.memory_history
            .iter()
            .map(|p| p.value)
            .fold(0.0, f32::max)
    }
}





