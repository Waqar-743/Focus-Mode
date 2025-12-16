use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockingErrorEvent {
    pub message: String,
}

#[derive(Clone, Default)]
pub struct FocusBlocker {
    inner: Arc<Mutex<Inner>>,
}

#[derive(Default)]
struct Inner {
    enabled: bool,
    toast_prev: Option<u32>,
    toast_prev_existed: bool,
}

const HOSTS_START: &str = "# focus-timer-block-start";
const HOSTS_END: &str = "# focus-timer-block-end";

fn blocked_domains() -> &'static [&'static str] {
    &[
        "facebook.com",
        "www.facebook.com",
        "m.facebook.com",
        "messenger.com",
        "www.messenger.com",
        "youtube.com",
        "www.youtube.com",
        "m.youtube.com",
        "youtu.be",
        "twitter.com",
        "www.twitter.com",
        "x.com",
        "www.x.com",
        "instagram.com",
        "www.instagram.com",
        "reddit.com",
        "www.reddit.com",
        "old.reddit.com",
    ]
}

fn hosts_path() -> PathBuf {
    let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
    PathBuf::from(system_root)
        .join("System32")
        .join("drivers")
        .join("etc")
        .join("hosts")
}

fn detect_newline(text: &str) -> &'static str {
    if text.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

fn strip_hosts_block(existing: &str) -> String {
    if !existing.contains(HOSTS_START) {
        return existing.to_string();
    }

    let newline = detect_newline(existing);

    let mut out = String::with_capacity(existing.len());
    let mut skipping = false;

    for line in existing.lines() {
        if line.trim_end() == HOSTS_START {
            skipping = true;
            continue;
        }
        if skipping {
            if line.trim_end() == HOSTS_END {
                skipping = false;
            }
            continue;
        }
        out.push_str(line);
        out.push_str(newline);
    }

    out
}

fn append_hosts_block(existing: &str) -> String {
    let cleaned = strip_hosts_block(existing);
    let newline = detect_newline(existing);

    let mut out = cleaned;

    if !out.ends_with('\n') && !out.is_empty() {
        out.push_str(newline);
    }

    out.push_str(HOSTS_START);
    out.push_str(newline);
    for domain in blocked_domains() {
        out.push_str("0.0.0.0 ");
        out.push_str(domain);
        out.push_str(newline);
    }
    out.push_str(HOSTS_END);
    out.push_str(newline);
    out
}

fn read_to_string(path: &Path) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|e| format!("failed to read {path:?}: {e}"))
}

fn write_string(path: &Path, contents: &str) -> Result<(), String> {
    fs::write(path, contents).map_err(|e| format!("failed to write {path:?}: {e}"))
}

#[cfg(target_os = "windows")]
fn set_toast_enabled(enabled: bool) -> Result<(Option<u32>, bool), String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\PushNotifications")
        .map_err(|e| format!("failed to open PushNotifications registry key: {e}"))?;

    let existed = key.get_raw_value("ToastEnabled").is_ok();
    let prev = key.get_value::<u32, _>("ToastEnabled").ok();

    let value: u32 = if enabled { 1 } else { 0 };
    key.set_value("ToastEnabled", &value)
        .map_err(|e| format!("failed to set ToastEnabled registry value: {e}"))?;

    Ok((prev, existed))
}

#[cfg(not(target_os = "windows"))]
fn set_toast_enabled(_enabled: bool) -> Result<(Option<u32>, bool), String> {
    Ok((None, false))
}

#[cfg(target_os = "windows")]
fn restore_toast_enabled(prev: Option<u32>, existed: bool) -> Result<(), String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\PushNotifications")
        .map_err(|e| format!("failed to open PushNotifications registry key: {e}"))?;

    if existed {
        if let Some(v) = prev {
            key.set_value("ToastEnabled", &v)
                .map_err(|e| format!("failed to restore ToastEnabled registry value: {e}"))?;
        }
    } else {
        let _ = key.delete_value("ToastEnabled");
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn restore_toast_enabled(_prev: Option<u32>, _existed: bool) -> Result<(), String> {
    Ok(())
}

impl FocusBlocker {
    pub fn ensure_disabled(&self) {
        let _ = self.disable();
    }

    pub fn enable(&self) -> Result<(), String> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| "focus blocker mutex poisoned".to_string())?;

        if inner.enabled {
            return Ok(());
        }

        // Suppress toasts (best effort) to reduce mail/message notifications.
        let (prev, existed) = set_toast_enabled(false)?;
        inner.toast_prev = prev;
        inner.toast_prev_existed = existed;

        // Block social sites via hosts file. This requires admin on Windows.
        let path = hosts_path();
        match read_to_string(&path)
            .map(|existing| append_hosts_block(&existing))
            .and_then(|updated| write_string(&path, &updated))
        {
            Ok(()) => {}
            Err(e) => {
                // Don't leave system notifications disabled if we couldn't enforce blocking.
                let _ = restore_toast_enabled(inner.toast_prev, inner.toast_prev_existed);
                inner.toast_prev = None;
                inner.toast_prev_existed = false;
                return Err(e);
            }
        }

        inner.enabled = true;
        Ok(())
    }

    pub fn disable(&self) -> Result<(), String> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| "focus blocker mutex poisoned".to_string())?;

        if !inner.enabled {
            // Still try to strip any leftover hosts section if present.
            let path = hosts_path();
            if let Ok(existing) = read_to_string(&path) {
                if existing.contains(HOSTS_START) {
                    let updated = strip_hosts_block(&existing);
                    let _ = write_string(&path, &updated);
                }
            }
            return Ok(());
        }

        // Restore notifications first so we don't keep the system muted on partial failures.
        restore_toast_enabled(inner.toast_prev, inner.toast_prev_existed)?;
        inner.toast_prev = None;
        inner.toast_prev_existed = false;

        let path = hosts_path();
        let existing = read_to_string(&path)?;
        let updated = strip_hosts_block(&existing);
        write_string(&path, &updated)?;

        inner.enabled = false;
        Ok(())
    }
}

pub fn emit_blocking_error(app: &AppHandle, message: impl Into<String>) {
    let _ = app.emit(
        "blocking-error",
        BlockingErrorEvent {
            message: message.into(),
        },
    );
}
