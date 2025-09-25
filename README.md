# netrs

Wayland-native Rust frontend for NetworkManager. Provides a GTK4 UI and a D-Bus proxy core, built entirely in Rust.
<br>
<br>
netrs is split into two components:
* **netrs-core**: D-Bus proxy, configuration, data models
* **netrs-ui**: GTK4 frontend, consumes `netrs-core`

## Dependencies

### Core

* `zbus` / `zvariant`: D-Bus communication
* `serde`: serialization
* `thiserror`: error handling
* `tracing`: structured logging

### UI

* `gtk4` / `glib`: GTK4-based interface
* `tokio`: async runtime
* `netrs-core`: internal dependency
