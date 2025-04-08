Crankshaft TUI
A powerful terminal-based monitoring dashboard for distributed task execution engines.

Overview
Crankshaft TUI delivers a feature-rich, real-time terminal interface to monitor and manage distributed task execution. Whether you're running tasks on local machines, cloud services, or high-performance computing (HPC) environments, this tool provides an intuitive, keyboard-driven experience to keep track of tasks, backends, resource utilization, and system events—all in one place.

Features
Multi-view Interface: Seamlessly switch between dashboard, task list, and backend views.

Real-time Task Monitoring: Track task creation, execution, completion, and failure as they happen.

Backend Health Tracking: Monitor the health and utilization of execution backends.

Resource Visualization: View CPU and memory usage with interactive graphs.

Task Management: Dive into detailed task information and live logs.

Adaptive Layout: Responsive design that adjusts to your terminal size.

Event Timeline: See a chronological display of system events and notifications.

Installation
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

Keyboard Controls
Key	Function
d	Dashboard view
t	Tasks list view
b	Backends list view
Tab	Cycle through views
↑ / ↓ or k/j	Navigate lists
Enter	Select/view details
Esc	Return to previous view
p	Toggle pause/live updates
? or F1	Toggle help screen
q	Quit application
Architecture
Core Components
State Management

AppState: Central hub for tracking tasks and backends.

TaskState: Details on individual task status, progress, and resource usage.

BackendState: Tracks backend health, tasks, and utilization metrics.

Monitoring System

TaskMonitor: Asynchronously tracks task status changes.

BackendMonitor: Polls backends for health and resource metrics.

User Interface

Multiple views (Dashboard, TaskList, etc.).

Adaptive layouts for various terminal sizes.

Custom widgets for resource visualization.

Event Handling

Event-driven design for processing user input.

Clear separation between events and state updates.

Data Flow

Input Events → UI Controller → State Updates → UI Rendering
↑                                ↑
|                                |
└─ Task / Backend Monitors ------┘

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
        
examples/
└── demo.rs           # Demo application
Customization
Themes
Tweak the UI look by creating a custom theme. For example:

rust
let mut ui = Ui::new();
let custom_theme = Theme {
    normal_text: Style::default().fg(Color::White),
    header_style: Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    // ... other style settings
};
ui.set_theme(custom_theme);
Connection to Real Backends
While the demo uses simulated data, you can connect to your real backend systems:

rust
// Initialize monitors with real endpoints
task_monitor.connect("http://your-crankshaft-engine:8080/tasks").await?;
backend_monitor.connect("http://your-crankshaft-engine:8080/backends").await?;
Development
Building and Testing
bash
# Run tests
cargo test

# Run with debug output
RUST_LOG=debug cargo run --example demo

# Generate documentation
cargo doc --open
Contributing
Fork the repository.

Create a new branch:
git checkout -b feature-branch
Commit your changes:

bash
git commit -m "Add some feature"
Push to the branch:

bash
git push origin feature-branch
Open a pull request.
