mod apigroup;
pub mod client;
mod discovery;
pub mod dynamic_object;
pub mod watch;
pub mod watch_event;
pub mod tekton_metrics;

pub use client::client;
pub use discovery::new;
pub use discovery::{dynamic_api, resolve_api_resources};
pub use watch::watch;
pub use watch_event::WatchEvent;
