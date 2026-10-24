//! Configuration options and builder for running the background daemon.

/// Configuration options for running the background daemon.
///
/// Follows the builder pattern and encapsulates polling intervals and feature toggles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DaemonConfig {
    /// Telemetry polling interval in seconds (default: 10).
    pub interval_secs: u64,
    /// Whether desktop notifications are enabled via `notify-send`.
    pub enable_notify: bool,
    /// Whether auto-profile switching is enabled.
    pub auto_profile: bool,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            interval_secs: 10,
            enable_notify: true,
            auto_profile: true,
        }
    }
}

impl DaemonConfig {
    /// Creates a new `DaemonConfig` with standard default settings.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            interval_secs: 10,
            enable_notify: true,
            auto_profile: true,
        }
    }

    /// Sets the polling interval in seconds.
    #[must_use]
    pub const fn with_interval(mut self, secs: u64) -> Self {
        self.interval_secs = secs;
        self
    }

    /// Enables or disables desktop notifications.
    #[must_use]
    pub const fn with_notifications(mut self, enable: bool) -> Self {
        self.enable_notify = enable;
        self
    }

    /// Enables or disables per-app auto-profile switching.
    #[must_use]
    pub const fn with_auto_profile(mut self, enable: bool) -> Self {
        self.auto_profile = enable;
        self
    }
}
