use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// Get the path to Claude Code's user settings file.
pub fn claude_settings_path() -> PathBuf {
    let home = dirs::home_dir().expect("Could not determine home directory");
    home.join(".claude").join("settings.json")
}

/// Get the path to the railyard binary.
fn railyard_binary_path() -> String {
    std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "railyard".to_string())
}

/// Install railyard hooks into Claude Code settings.
pub fn install_hooks() -> Result<String, String> {
    let settings_path = claude_settings_path();
    let mut settings = read_settings(&settings_path)?;
    let binary = railyard_binary_path();

    let hooks = settings
        .as_object_mut()
        .ok_or("Settings is not a JSON object")?
        .entry("hooks")
        .or_insert_with(|| json!({}));

    let hooks_obj = hooks
        .as_object_mut()
        .ok_or("hooks is not a JSON object")?;

    // PreToolUse hook — blocks/approves/traces before execution
    let pre_hook = json!([{
        "matcher": "",
        "hooks": [{
            "type": "command",
            "command": format!("{} hook --event PreToolUse", binary),
            "timeout": 5
        }]
    }]);

    // PostToolUse hook — traces results, captures snapshots
    let post_hook = json!([{
        "matcher": "",
        "hooks": [{
            "type": "command",
            "command": format!("{} hook --event PostToolUse", binary),
            "timeout": 5
        }]
    }]);

    // SessionStart hook — initializes session logging
    let session_hook = json!([{
        "matcher": "",
        "hooks": [{
            "type": "command",
            "command": format!("{} hook --event SessionStart", binary),
            "timeout": 5
        }]
    }]);

    hooks_obj.insert("PreToolUse".to_string(), pre_hook);
    hooks_obj.insert("PostToolUse".to_string(), post_hook);
    hooks_obj.insert("SessionStart".to_string(), session_hook);

    write_settings(&settings_path, &settings)?;

    Ok(format!(
        "Installed railyard hooks in {}",
        settings_path.display()
    ))
}

/// Remove railyard hooks from Claude Code settings.
pub fn uninstall_hooks() -> Result<String, String> {
    let settings_path = claude_settings_path();

    if !settings_path.exists() {
        return Ok("No Claude Code settings found, nothing to uninstall".to_string());
    }

    let mut settings = read_settings(&settings_path)?;

    if let Some(hooks) = settings.get_mut("hooks").and_then(|h| h.as_object_mut()) {
        // Remove only hooks that reference railyard
        for event in &["PreToolUse", "PostToolUse", "SessionStart"] {
            if let Some(event_hooks) = hooks.get_mut(*event) {
                if let Some(arr) = event_hooks.as_array_mut() {
                    arr.retain(|entry| {
                        let is_railyard = entry
                            .pointer("/hooks/0/command")
                            .and_then(|c| c.as_str())
                            .is_some_and(|c| c.contains("railyard"));
                        !is_railyard
                    });
                    if arr.is_empty() {
                        hooks.remove(*event);
                    }
                }
            }
        }
    }

    write_settings(&settings_path, &settings)?;

    Ok(format!(
        "Removed railyard hooks from {}",
        settings_path.display()
    ))
}

/// Check if railyard hooks are currently installed.
pub fn check_installed() -> Result<bool, String> {
    let settings_path = claude_settings_path();
    if !settings_path.exists() {
        return Ok(false);
    }

    let settings = read_settings(&settings_path)?;

    if let Some(hooks) = settings.get("hooks").and_then(|h| h.as_object()) {
        if let Some(pre) = hooks.get("PreToolUse").and_then(|v| v.as_array()) {
            for entry in pre {
                if let Some(cmd) = entry.pointer("/hooks/0/command").and_then(|c| c.as_str()) {
                    if cmd.contains("railyard") {
                        return Ok(true);
                    }
                }
            }
        }
    }

    Ok(false)
}

fn read_settings(path: &Path) -> Result<Value, String> {
    if !path.exists() {
        // Create parent directory and return empty object
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create settings dir: {}", e))?;
        }
        return Ok(json!({}));
    }

    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read settings: {}", e))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings: {}", e))
}

fn write_settings(path: &Path, settings: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create settings dir: {}", e))?;
    }

    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    fs::write(path, content).map_err(|e| format!("Failed to write settings: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_path_exists() {
        let path = claude_settings_path();
        assert!(path.to_str().unwrap().contains(".claude"));
        assert!(path.to_str().unwrap().ends_with("settings.json"));
    }
}
