# Crankshaft TUI

A powerful terminal-based monitoring dashboard for distributed task execution engines.

## Overview

Crankshaft TUI delivers a feature-rich, real-time terminal interface to monitor and manage distributed task execution. Whether you're running tasks on local machines, cloud services, or high-performance computing (HPC) environments, this tool provides an intuitive, keyboard-driven experience to keep track of tasks, backends, resource utilization, and system events—all in one place.

## Features

- **Multi-view Interface**: Seamlessly switch between dashboard, task list, and backend views.
- **Real-time Task Monitoring**: Track task creation, execution, completion, and failure as they happen.
- **Backend Health Tracking**: Monitor the health and utilization of execution backends.
- **Resource Visualization**: View CPU and memory usage with interactive graphs.
- **Task Management**: Dive into detailed task information and live logs.
- **Adaptive Layout**: Responsive design that adjusts to your terminal size.
- **Event Timeline**: See a chronological display of system events and notifications.

## Installation

### Prerequisites

- Rust 1.70.0 or newer
- Cargo package manager

### Building from Source

Clone the repository and build:
git clone https://github.com/blenbot/crankshaft-tui.git
cd crankshaft-tui
cargo build --release


## Usage

### Running the Demo

To quickly explore Crankshaft TUI, launch the demo:
cargo run --example demo


Prerequisites
Rust 1.70.0 or newer

Cargo package manager

Building from Source
bash
# Clone the repository
git clone https://github.com/yourusername/crankshaft-tui.git
cd crankshaft-tui

# Build the project
cargo build --release
Usage
Running the Demo
The fastest way to explore Crankshaft TUI is by launching the included demo:

bash
cargo run --example demo
This spins up a fully functional interface with simulated task and backend data to play with.

## Architecture

### Core Components

- **State Management**
  - *AppState*: Central hub for tracking tasks and backends.
  - *TaskState*: Maintains details on individual task status, progress, and resource usage.
  - *BackendState*: Tracks backend health, tasks, and utilization metrics.

- **Monitoring System**
  - *TaskMonitor*: Asynchronously tracks task status changes.
  - *BackendMonitor*: Polls backends for health and resource metrics.

- **User Interface**
  - Multiple views (Dashboard, TaskList, etc.) with adaptive layouts.
  - Custom widgets for resource visualization.

- **Event Handling**
  - Event-driven design for processing user input.
  - Clear separation between events and state updates.

- **Data Flow**

  Input Events → UI Controller → State Updates → UI Rendering  
  ↑                             ↑  
  |                             |  
  └──── Task/Backend Monitors  ─┘

Project Structure
text
src/
├── app.rs            # Application logic
├── event/            # Event handling
│   ├── handler.rs
│   └── mod.rs
├── main.rs           # Program entry point
├── monitor/          # Task and backend monitoring
│   ├── backend.rs
│   ├── mod.rs
│   └── task.rs
├── state/            # Application state management
│   ├── backend.rs
│   ├── mod.rs
│   ├── resource.rs
│   └── task.rs
└── ui/               # User interface components
    ├── backend_view.rs
    ├── dashboard.rs
    ├── help.rs
    ├── log_view.rs
    ├── mod.rs
    ├── task_detail.rs
    ├── task_list.rs
    ├── theme.ts
    └── widgets/
        ├── mod.rs
        ├── progress.rs
        ├── sparkline.rs
        ├── stat_panel.rs
        └── tabbed_view.rs
        
# Generate documentation
cargo doc --open
