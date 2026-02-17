//! Skill tools: list, search, install, info, enable, link_secret, publish, remove, update.
//!
//! These are gateway-intercepted tools. In standalone mode they provide
//! helpful guidance. In gateway mode the gateway uses its SkillManager.

use serde_json::Value;
use std::path::Path;

/// List all loaded skills with their status.
pub fn exec_skill_list(_args: &Value, _workspace_dir: &Path) -> Result<String, String> {
    // Stub — the gateway intercepts this and uses its SkillManager.
    Ok("No skills loaded (standalone mode). Connect to the gateway for full skill support.".into())
}

/// Search the ClawHub registry for installable skills.
pub fn exec_skill_search(args: &Value, _workspace_dir: &Path) -> Result<String, String> {
    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: query".to_string())?;

    Ok(format!(
        "To search for skills matching '{}':\n\n\
         Use the CLI: rustyclaw skills search \"{}\"\n\
         Or browse skills at: https://clawhub.ai",
        query, query,
    ))
}

/// Install a skill from the ClawHub registry.
pub fn exec_skill_install(args: &Value, _workspace_dir: &Path) -> Result<String, String> {
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: name".to_string())?;

    Ok(format!(
        "To install the '{}' skill:\n\n\
         rustyclaw skills install {}\n\n\
         The skill will be installed to your workspace/skills directory.",
        name, name,
    ))
}

/// Show detailed information about a loaded skill.
pub fn exec_skill_info(args: &Value, _workspace_dir: &Path) -> Result<String, String> {
    let _name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: name".to_string())?;

    Ok("Skill info requires gateway connection for full details.".into())
}

/// Enable or disable a loaded skill.
pub fn exec_skill_enable(args: &Value, _workspace_dir: &Path) -> Result<String, String> {
    let _name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: name".to_string())?;
    let _enabled = args
        .get("enabled")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Missing required parameter: enabled".to_string())?;

    Err("Skill enable/disable requires gateway connection.".into())
}

/// Link or unlink a vault credential to a skill.
pub fn exec_skill_link_secret(args: &Value, _workspace_dir: &Path) -> Result<String, String> {
    let action = args
        .get("action")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: action".to_string())?;
    let _skill = args
        .get("skill")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: skill".to_string())?;
    let _secret = args
        .get("secret")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: secret".to_string())?;

    if !matches!(action, "link" | "unlink") {
        return Err(format!(
            "Unknown action '{}'. Use 'link' or 'unlink'.",
            action
        ));
    }

    Err("Skill secret linking requires gateway connection.".into())
}

/// Publish a local skill to the ClawHub registry.
pub fn exec_skill_publish(args: &Value, _workspace_dir: &Path) -> Result<String, String> {
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: name".to_string())?;

    Ok(format!(
        "To publish skill '{}':\n\n\
         rustyclaw skills publish {}\n\n\
         Requires clawhub_token set in config.toml.",
        name, name,
    ))
}

/// Remove an installed skill.
pub fn exec_skill_remove(args: &Value, _workspace_dir: &Path) -> Result<String, String> {
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: name".to_string())?;

    Ok(format!(
        "To remove skill '{}':\n\n\
         rustyclaw skills remove {}",
        name, name,
    ))
}

/// Update a registry-installed skill to the latest version.
pub fn exec_skill_update(args: &Value, _workspace_dir: &Path) -> Result<String, String> {
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: name".to_string())?;

    Ok(format!(
        "To update skill '{}':\n\n\
         rustyclaw skills update {}",
        name, name,
    ))
}
