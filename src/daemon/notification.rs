//! Desktop notification triggers and alerts for battery level and auto-profile changes.

use std::process::Command;
use std::time::{Duration, Instant};

/// Send a native desktop notification via `notify-send`.
pub fn send_desktop_notification(summary: &str, body: &str, urgency: &str) {
    let _ = Command::new("notify-send")
        .arg("-a")
        .arg("ICARUS V2 MINI")
        .arg("-u")
        .arg(urgency)
        .arg("-i")
        .arg("input-mouse")
        .arg(summary)
        .arg(body)
        .status();
}

/// Evaluates battery telemetry and emits desktop notification when threshold triggers.
pub fn check_and_notify_battery(
    level: u8,
    is_charging: bool,
    last_charging_state: &mut bool,
    last_notified_low: &mut Option<Instant>,
) {
    // Low battery alert with 5-minute cooldown
    if level <= 15 && !is_charging {
        let should_notify = match *last_notified_low {
            None => true,
            Some(t) => t.elapsed() >= Duration::from_secs(300),
        };
        if should_notify {
            send_desktop_notification(
                "⚠️ ICARUS V2 MINI: Baterai Lemah!",
                &format!(
                    "Sisa baterai mouse {}%. Harap sambungkan kabel USB pengisi daya.",
                    level
                ),
                "critical",
            );
            *last_notified_low = Some(Instant::now());
        }
    }

    // Fully charged alert (triggers once when battery reaches 100% while charging)
    let is_full_while_charging = is_charging && level >= 100;
    if !*last_charging_state && is_full_while_charging {
        send_desktop_notification(
            "🔋 ICARUS V2 MINI: Baterai Penuh!",
            "Baterai mouse telah terisi 100%. Kabel dapat dilepas.",
            "normal",
        );
    }

    *last_charging_state = is_full_while_charging;
}
