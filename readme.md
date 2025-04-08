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
'''
git clone https://github.com/blenbot/crankshaft-tui.git

cd crankshaft-tui

cargo build --release
'''

## Usage

### Running the Demo

To quickly explore Crankshaft TUI, launch the demo:
'''
cargo run --example demo
'''

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

- **User Interface**
  - Multiple views (Dashboard, TaskList, etc.) with adaptive layouts.
  - Custom widgets for resource visualization.

- **Event Handling**
  - Event-driven design for processing user input.
  - Clear separation between events and state updates.
