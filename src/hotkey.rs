use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use handy_keys::{Hotkey, HotkeyManager, HotkeyState};
use log::{error, info, warn};

use crate::load_config;

/// Parse the hotkey from CLI args, env var, config file, or platform default.
/// Priority: --hotkey flag > HAMMERTALK_HOTKEY env > config file > platform default.
/// On macOS the default is "Fn" (globe key). On Linux there is no default
/// (users bind keys in their compositor via hammertalk-ctl).
/// Pass --hotkey none to disable the hotkey.
pub fn parse_hotkey_arg() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();

    // CLI arg takes priority
    if let Some(pos) = args.iter().position(|a| a == "--hotkey") {
        if let Some(val) = args.get(pos + 1) {
            if val.eq_ignore_ascii_case("none") {
                return None;
            }
            return Some(val.clone());
        }
    }

    // Env var
    if let Ok(val) = std::env::var("HAMMERTALK_HOTKEY") {
        if val.eq_ignore_ascii_case("none") {
            return None;
        }
        return Some(val);
    }

    // Config file
    let config = load_config();
    if let Some(hotkey) = config.hotkey {
        if hotkey.eq_ignore_ascii_case("none") {
            return None;
        }
        return Some(hotkey);
    }

    // Platform default: Fn on macOS, no default on Linux
    if cfg!(target_os = "macos") {
        Some("Fn".to_string())
    } else {
        None
    }
}

/// Check accessibility permissions (macOS) and log a helpful message if missing.
pub fn check_permissions() -> bool {
    #[cfg(target_os = "macos")]
    {
        if !handy_keys::check_accessibility() {
            error!("Accessibility permission required for global hotkeys.");
            error!("Grant it in: System Settings → Privacy & Security → Accessibility");
            if let Err(e) = handy_keys::open_accessibility_settings() {
                warn!("Could not open Accessibility settings: {}", e);
            }
            return false;
        }
    }
    true
}

/// Start a global hotkey listener that sends SIGUSR1 on press and SIGUSR2 on release.
/// This runs in the current thread and blocks forever.
pub fn run_hotkey_listener(hotkey_str: &str, running: Arc<AtomicBool>) {
    let hotkey: Hotkey = match hotkey_str.parse() {
        Ok(h) => h,
        Err(e) => {
            error!("Invalid hotkey '{}': {}", hotkey_str, e);
            error!("Examples: Cmd+Shift+T, Ctrl+Alt+Space, F18");
            return;
        }
    };

    let manager = match HotkeyManager::new() {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to create hotkey manager: {}", e);
            return;
        }
    };

    let id = match manager.register(hotkey) {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to register hotkey '{}': {}", hotkey_str, e);
            return;
        }
    };

    info!("Global hotkey registered: {} (id={:?})", hotkey, id);
    info!("Hold the key to record, release to transcribe.");

    let pid = std::process::id() as i32;

    while running.load(Ordering::SeqCst) {
        match manager.recv() {
            Ok(event) => {
                if event.id == id {
                    match event.state {
                        HotkeyState::Pressed => {
                            info!("Hotkey pressed → starting recording");
                            // Send SIGUSR1 to ourselves
                            unsafe {
                                libc::kill(pid, libc::SIGUSR1);
                            }
                        }
                        HotkeyState::Released => {
                            info!("Hotkey released → stopping recording");
                            // Send SIGUSR2 to ourselves
                            unsafe {
                                libc::kill(pid, libc::SIGUSR2);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                if running.load(Ordering::SeqCst) {
                    warn!("Hotkey listener error: {}", e);
                }
                break;
            }
        }
    }
}
