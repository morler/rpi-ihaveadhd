#![allow(clippy::not_unsafe_ptr_arg_deref)]
use rpi_plugin_sdk::{callbacks::free_string, StbStringRef};
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

const RULES: &str = include_str!("adhd.md");
static ENABLED: AtomicBool = AtomicBool::new(false);
static INIT: OnceLock<()> = OnceLock::new();

fn agent_dir(home: &Path, agent_dir: Option<&OsStr>) -> PathBuf {
    agent_dir
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".rpi").join("agent"))
}

fn config_path() -> Option<PathBuf> {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)?;
    Some(agent_dir(&home, std::env::var_os("RPI_AGENT_DIR").as_deref()).join("rpi-adhd.json"))
}

/// None when RPI_ADHD is unset or unparsable.
fn switch_value(raw: &str) -> Option<bool> {
    match raw.trim().to_lowercase().as_str() {
        "1" | "on" | "true" => Some(true),
        "0" | "off" | "false" => Some(false),
        _ => None,
    }
}

/// None when RPI_ADHD is unset or unparsable.
fn env_switch() -> Option<bool> {
    switch_value(&std::env::var("RPI_ADHD").unwrap_or_default())
}

fn parse_enabled(content: &str) -> Option<bool> {
    serde_json::from_str::<serde_json::Value>(content.trim_start_matches('﻿'))
        .ok()
        .and_then(|v| v["enabled"].as_bool())
}

fn file_enabled(path: &Path) -> Option<bool> {
    parse_enabled(&std::fs::read_to_string(path).ok()?)
}

fn default_enabled() -> bool {
    env_switch()
        .or_else(|| config_path().and_then(|path| file_enabled(&path)))
        .unwrap_or(false)
}

fn enabled() -> bool {
    INIT.get_or_init(|| ENABLED.store(default_enabled(), Ordering::Relaxed));
    ENABLED.load(Ordering::Relaxed)
}

/// Merge `enabled` into the existing JSON object and replace the file atomically,
/// so a failed write never leaves a half-written config behind.
fn save_enabled(path: &Path, enabled: bool) -> io::Result<()> {
    let mut config = match std::fs::read_to_string(path) {
        Ok(content) => serde_json::from_str::<serde_json::Value>(content.trim_start_matches('﻿'))
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
        Err(e) if e.kind() == io::ErrorKind::NotFound => serde_json::json!({}),
        Err(e) => return Err(e),
    };
    let object = config
        .as_object_mut()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "config is not a JSON object"))?;
    object.insert("enabled".into(), enabled.into());
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "config has no parent"))?;
    std::fs::create_dir_all(parent)?;
    let temp = parent.join(format!("config.json.{}.tmp", std::process::id()));
    let mut file = std::fs::File::create(&temp)?;
    serde_json::to_writer_pretty(&mut file, &config)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    drop(file);
    std::fs::rename(&temp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&temp);
    })
}

fn saved_message(enabled: bool, env: Option<bool>) -> String {
    let state = if enabled { "on" } else { "off" };
    match env.filter(|env| *env != enabled) {
        Some(env) => format!(
            "ADHD mode {state} and saved. RPI_ADHD keeps the startup default at {}.",
            if env { "on" } else { "off" }
        ),
        None => format!("ADHD mode {state} and saved."),
    }
}

fn save_default(enabled: bool) -> io::Result<String> {
    let path = config_path()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no rpi-adhd config directory"))?;
    save_enabled(&path, enabled)?;
    Ok(saved_message(enabled, env_switch()))
}

/// Returns the next state (when it changes) and the reply text.
fn command(args: &str, current: bool) -> (Option<bool>, String) {
    match args.trim().to_lowercase().as_str() {
        "" => (Some(!current), String::new()),
        "on" | "start" => (Some(true), String::new()),
        "off" | "stop" => (Some(false), String::new()),
        "status" => (
            None,
            format!(
                "ADHD mode: {} • saved default: {}",
                if current { "on" } else { "off" },
                if default_enabled() { "on" } else { "off" }
            ),
        ),
        _ => (None, "Usage: /adhd [on|off|status]".into()),
    }
}

/// Replacement, not append: the host uses the returned text as the whole prompt.
fn augment(base: &str) -> String {
    if base.is_empty() {
        RULES.to_owned()
    } else {
        format!("{base}\n\n{RULES}")
    }
}

rpi_plugin_sdk::string_handler! {
    fn on_command(input, out, _user_data) {
        let value: serde_json::Value = serde_json::from_str(input).unwrap_or_default();
        let args = value["args"].as_str().unwrap_or("");
        let (next, mut text) = command(args, enabled());
        if let Some(next) = next {
            match save_default(next) {
                Ok(message) => {
                    ENABLED.store(next, Ordering::Relaxed);
                    text = message;
                }
                Err(e) => text = format!("ADHD mode not saved: {e}"),
            }
        }
        i32::from(!out.write(serde_json::json!({"kind":"message", "text":text}).to_string()))
    }
}
rpi_plugin_sdk::string_handler! {
    fn on_start(input, out, _user_data) {
        let value: serde_json::Value = match serde_json::from_str(input) { Ok(v) => v, Err(_) => return 1 };
        if !enabled() { return i32::from(!out.write("{}".to_string())); }
        let prompt = augment(value["systemPrompt"].as_str().unwrap_or(""));
        i32::from(!out.write(serde_json::json!({"systemPrompt":prompt}).to_string()))
    }
}
rpi_plugin_sdk::export_plugin!(|api| {
    let Some(register_prompt) = api.register_before_agent_start else {
        return 1;
    };
    let Some(register_command) = api.register_command else {
        return 1;
    };
    let rc = register_prompt(on_start, free_string, std::ptr::null_mut());
    if rc != 0 {
        return rc;
    }
    register_command(
        StbStringRef::from_str("adhd"),
        StbStringRef::from_str("ADHD-friendly output: on, off, status"),
        on_command,
    )
});

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("rpi-adhd-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn toggles_and_reports() {
        assert_eq!(command("", false).0, Some(true));
        assert_eq!(command(" ON ", false).0, Some(true));
        assert_eq!(command("off", true).0, Some(false));
        assert_eq!(command("stop", true).0, Some(false));
        assert!(command("status", true).1.contains("ADHD mode: on"));
        assert!(command("maybe", true).0.is_none());
        assert!(command("usage", true).1.starts_with("Usage"));
    }

    #[test]
    fn prompt_keeps_the_base_and_appends_the_rules() {
        assert!(augment("BASE").starts_with("BASE\n\nADHD MODE ACTIVE"));
        assert!(augment("").starts_with("ADHD MODE ACTIVE"));
    }

    #[test]
    fn saves_and_reads_back_keeping_other_keys() {
        let dir = temp_dir("save");
        let path = dir.join("rpi-adhd.json");
        save_enabled(&path, true).unwrap();
        assert_eq!(file_enabled(&path), Some(true));

        std::fs::write(&path, r#"{"enabled":false,"quietStartup":true}"#).unwrap();
        save_enabled(&path, true).unwrap();
        let saved: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(saved["enabled"], true);
        assert_eq!(saved["quietStartup"], true);
        assert_eq!(
            std::fs::read_dir(path.parent().unwrap()).unwrap().count(),
            1
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn invalid_config_is_reported_and_left_untouched() {
        let dir = temp_dir("invalid");
        let path = dir.join("rpi-adhd.json");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        for bad in ["not json", "[1,2]"] {
            std::fs::write(&path, bad).unwrap();
            assert!(save_enabled(&path, true).is_err());
            assert_eq!(std::fs::read_to_string(&path).unwrap(), bad);
        }
        assert_eq!(parse_enabled("\u{feff}{\"enabled\": true}"), Some(true));
        assert_eq!(parse_enabled(r#"{"enabled":"yes"}"#), None);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn rules_are_single_sourced_from_the_markdown() {
        assert!(RULES.contains("第一行就是下一步动作"));
        assert!(!RULES.starts_with("---"));
    }

    #[test]
    fn config_lives_in_the_agent_dir() {
        let home = Path::new("/h");
        assert_eq!(agent_dir(home, None), home.join(".rpi/agent"));
        assert_eq!(agent_dir(home, Some(OsStr::new("/a"))), Path::new("/a"));
        assert_eq!(
            agent_dir(home, Some(OsStr::new(""))),
            home.join(".rpi/agent")
        );
    }

    #[test]
    fn switch_value_accepts_every_spelling() {
        for on in ["1", "on", "true", " ON ", "True"] {
            assert_eq!(switch_value(on), Some(true), "{on}");
        }
        for off in ["0", "off", "false", " OFF ", "False"] {
            assert_eq!(switch_value(off), Some(false), "{off}");
        }
        assert_eq!(switch_value(""), None);
        assert_eq!(switch_value("maybe"), None);
    }

    #[test]
    fn missing_or_unreadable_config_settles_the_default() {
        let dir = temp_dir("missing");
        let path = dir.join("rpi-adhd.json");
        assert_eq!(file_enabled(&path), None);
        // A directory where the config should be: the read fails, the write must too.
        std::fs::create_dir_all(&path).unwrap();
        assert_eq!(file_enabled(&path), None);
        assert!(save_enabled(&path, true).is_err());
        // Initializes the cached flag; its value depends on this machine's config.
        let _ = enabled();
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn saved_message_names_a_conflicting_env_default() {
        assert_eq!(saved_message(true, None), "ADHD mode on and saved.");
        assert_eq!(saved_message(false, None), "ADHD mode off and saved.");
        assert_eq!(saved_message(true, Some(true)), "ADHD mode on and saved.");
        assert!(
            saved_message(true, Some(false)).contains("RPI_ADHD keeps the startup default at off")
        );
        assert!(
            saved_message(false, Some(true)).contains("RPI_ADHD keeps the startup default at on")
        );
    }
}
