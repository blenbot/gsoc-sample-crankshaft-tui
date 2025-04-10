# Crankshaft TUI

A powerful terminal-based monitoring dashboard for distributed task execution engines.

## Overview

Crankshaft TUI delivers a feature-rich, real-time terminal interface to monitor and manage distributed task execution. Whether you're running tasks on local machines, cloud services, or high-performance computing (HPC) environments, this tool provides an intuitive, keyboard-driven experience to keep track of tasks, backends, resource utilization, and system events—all in one place.

## Features

**Multi-view Interface**: Seamlessly switch between dashboard, task list, and backend views
**Real-time Task Monitoring**: Track task creation, execution, completion, and failure as they happen
**Backend Health Tracking**: Monitor the health and utilization of execution backends
**Resource Visualization**: View CPU and memory usage with interactive graphs
**Task Management**: Dive into detailed task information and live logs
**Adaptive Layout**: Responsive design that adjusts to your terminal size
**Event Timeline**: See a chronological display of system events and notifications
**Scale-Ready**: I created this project while keeping in mind scalability which crankshaft will require to handle workflow upto 20,000.

## Installation

### Prerequisites

- Rust 1.70.0 or newer
- Cargo package manager

### Building from Source

Clone the repository and build:

```
git clone https://github.com/blenbot/crankshaft-tui.git

cd crankshaft-tui

cargo build --release
```

## Usage

### Running the Demo

To quickly explore Crankshaft TUI, launch the demo:

```
cargo run --example demo
```

Prerequisites:

Rust 1.70.0 or newer

Cargo package manager

## Architecture

### Core Components

- **State Management**
  - *AppState*: Central hub for tracking tasks and backends.
  - *TaskState*: Maintains details on individual task status, progress, and resource usage.
  - *BackendState*: Tracks backend health, tasks, and utilization metrics.

- **Monitoring System**
  - *TaskMonitor*: Asynchronously tracks task status changes.
  - *BackendMonitor*: Polls backends for health and resource metrics.
  - Partially uses Tokio to periodically poll Crankshaft for updates (every few seconds), currently only simulated data not connected to actual crankshaft engine which will be one of the implementation for GSOC 2025.

- **User Interface**
 - *Note*: Currently it acts as boilerplate but most of these features are implemented to a good extent
 - *Dashboard View*: Global overview showing task summary, backend health, and recent events
 - *Task List View*: Detailed list of all tasks with filtering and sorting
 - *Backend View*: Backend-specific details with health metrics and task distribution
 - *Detail Views*: In-depth information about specific tasks or backends with tabbed navigation

- **Event Handling**
  - Event-driven design for processing user input.
  - Clear separation between events and state updates.


## Implementation:
 - **Backend Integration:**
  The current implementation includes three different backend types:
   - *Docker*: For container-based task execution
   - *TES (Task Execution Service)*: For cloud-based execution
   - *Local Runner*: For direct execution on the host system
   - Each backend exposes metrics for utilization, task count, and health status. The system is designed to be extensible for adding more backend types in the future.

 - **Data Visualization:**
  Custom widgets provide rich visualization of:
   - Task status distribution (bar charts)
   - Resource utilization over time (sparklines)
   - Task progress (progress bars)
   - Health status (color-coded indicators)
 - **Adaptive UI:**
   The interface dynamically adjusts to terminal size, reorganizing components to maintain usability even on smaller screens.

## Future Roadmap
 Based on project requirements, these features are planned for future development:
   - *Log Integration*: Modifying Crankshaft to expose logs through a Backend trait method
   - *Structured Logging*: Implementing a consistent log format across different backends
   - *Network Monitoring*: Adding network stream visualization alongside CPU and memory
   - *Scale Optimization*: Further optimizations for handling 20,000+ workflows efficiently
   - *Real-time Notifications*: Alert system for critical events
   - *Custom Filtering*: Advanced task and backend filtering options