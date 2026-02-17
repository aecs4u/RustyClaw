use super::SharedSkillManager;

/// Dispatch a skill management tool call.
///
/// Like `execute_secrets_tool`, these tools bypass the normal
/// `tools::execute_tool` path because they need access to the shared
/// `SkillManager` that lives in the gateway process.
pub async fn execute_skill_tool(
    name: &str,
    args: &serde_json::Value,
    skill_mgr: &SharedSkillManager,
) -> Result<String, String> {
    match name {
        "skill_list" => exec_gw_skill_list(args, skill_mgr).await,
        "skill_search" => exec_gw_skill_search(args, skill_mgr).await,
        "skill_install" => exec_gw_skill_install(args, skill_mgr).await,
        "skill_info" => exec_gw_skill_info(args, skill_mgr).await,
        "skill_enable" => exec_gw_skill_enable(args, skill_mgr).await,
        "skill_link_secret" => exec_gw_skill_link_secret(args, skill_mgr).await,
        "skill_publish" => exec_gw_skill_publish(args, skill_mgr).await,
        "skill_remove" => exec_gw_skill_remove(args, skill_mgr).await,
        "skill_update" => exec_gw_skill_update(args, skill_mgr).await,
        _ => Err(format!("Unknown skill tool: {}", name)),
    }
}

/// List all loaded skills, optionally filtered.
pub async fn exec_gw_skill_list(
    args: &serde_json::Value,
    skill_mgr: &SharedSkillManager,
) -> Result<String, String> {
    let filter = args
        .get("filter")
        .and_then(|v| v.as_str())
        .unwrap_or("all");

    let mgr = skill_mgr.lock().await;
    let skills = mgr.get_skills();

    if skills.is_empty() {
        return Ok("No skills loaded.".into());
    }

    let filtered: Vec<_> = skills
        .iter()
        .filter(|s| match filter {
            "enabled" => s.enabled,
            "disabled" => !s.enabled,
            "registry" => matches!(s.source, crate::skills::SkillSource::Registry { .. }),
            _ => true, // "all"
        })
        .collect();

    if filtered.is_empty() {
        return Ok(format!("No skills match filter '{}'.", filter));
    }

    let mut lines = Vec::with_capacity(filtered.len() + 1);
    lines.push(format!("{} skill(s):\n", filtered.len()));
    for s in &filtered {
        let status = if s.enabled { "✓" } else { "✗" };
        let source = match &s.source {
            crate::skills::SkillSource::Local => "local".to_string(),
            crate::skills::SkillSource::Registry { version, .. } => {
                format!("registry v{}", version)
            }
        };
        let secrets = if s.linked_secrets.is_empty() {
            String::new()
        } else {
            format!(" [secrets: {}]", s.linked_secrets.join(", "))
        };
        lines.push(format!(
            "  {} {} ({}) — {}{}\n",
            status,
            s.name,
            source,
            s.description.as_deref().unwrap_or("(no description)"),
            secrets,
        ));
    }
    Ok(lines.join(""))
}

/// Search the ClawHub registry.
pub async fn exec_gw_skill_search(
    args: &serde_json::Value,
    skill_mgr: &SharedSkillManager,
) -> Result<String, String> {
    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: query".to_string())?;

    let mgr = skill_mgr.lock().await;
    let results = mgr.search_registry(query).map_err(|e| e.to_string())?;

    if results.is_empty() {
        return Ok(format!("No skills found matching '{}'.", query));
    }

    let mut lines = Vec::with_capacity(results.len() + 1);
    lines.push(format!("{} result(s) for '{}':\n", results.len(), query));
    for r in &results {
        let secrets_note = if r.required_secrets.is_empty() {
            String::new()
        } else {
            format!(" (needs: {})", r.required_secrets.join(", "))
        };
        // Use display_name if available, otherwise name
        let display = if r.display_name.is_empty() { &r.name } else { &r.display_name };
        let version_str = if r.version.is_empty() { "latest".to_string() } else { format!("v{}", r.version) };
        lines.push(format!(
            "  • {} ({}) {} — {}{}\n",
            display, r.name, version_str, r.description, secrets_note,
        ));
    }
    lines.push("\nTo install: clawhub install <skill-name>".to_string());
    Ok(lines.join(""))
}

/// Install a skill from the ClawHub registry.
pub async fn exec_gw_skill_install(
    args: &serde_json::Value,
    skill_mgr: &SharedSkillManager,
) -> Result<String, String> {
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: name".to_string())?;
    let version = args.get("version").and_then(|v| v.as_str());

    let mut mgr = skill_mgr.lock().await;
    mgr.install_from_registry(name, version).map_err(|e| e.to_string())?;

    // Reload skills so the new one is available immediately.
    mgr.load_skills().map_err(|e| e.to_string())?;

    let version_note = version
        .map(|v| format!(" v{}", v))
        .unwrap_or_else(|| " (latest)".into());
    Ok(format!(
        "Skill '{}'{} installed from ClawHub and loaded.",
        name, version_note,
    ))
}

/// Show detailed information about a skill.
pub async fn exec_gw_skill_info(
    args: &serde_json::Value,
    skill_mgr: &SharedSkillManager,
) -> Result<String, String> {
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: name".to_string())?;

    let mgr = skill_mgr.lock().await;
    mgr.skill_info(name)
        .ok_or_else(|| format!("Skill '{}' not found.", name))
}

/// Enable or disable a skill.
pub async fn exec_gw_skill_enable(
    args: &serde_json::Value,
    skill_mgr: &SharedSkillManager,
) -> Result<String, String> {
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: name".to_string())?;
    let enabled = args
        .get("enabled")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Missing required parameter: enabled".to_string())?;

    let mut mgr = skill_mgr.lock().await;
    mgr.set_skill_enabled(name, enabled)
        .map_err(|e| e.to_string())?;

    let state = if enabled { "enabled" } else { "disabled" };
    Ok(format!("Skill '{}' is now {}.", name, state))
}

/// Publish a local skill to the ClawHub registry.
pub async fn exec_gw_skill_publish(
    args: &serde_json::Value,
    skill_mgr: &SharedSkillManager,
) -> Result<String, String> {
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: name".to_string())?;

    let mgr = skill_mgr.lock().await;
    mgr.publish_to_registry(name).map_err(|e| e.to_string())
}

/// Remove an installed skill.
pub async fn exec_gw_skill_remove(
    args: &serde_json::Value,
    skill_mgr: &SharedSkillManager,
) -> Result<String, String> {
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: name".to_string())?;

    let mut mgr = skill_mgr.lock().await;
    mgr.remove_skill(name).map_err(|e| e.to_string())?;
    Ok(format!("Skill '{}' removed.", name))
}

/// Update a registry-installed skill to the latest version.
pub async fn exec_gw_skill_update(
    args: &serde_json::Value,
    skill_mgr: &SharedSkillManager,
) -> Result<String, String> {
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: name".to_string())?;

    let mut mgr = skill_mgr.lock().await;
    match mgr.update_skill(name).map_err(|e| e.to_string())? {
        true => Ok(format!("Skill '{}' updated to latest version.", name)),
        false => Ok(format!("Skill '{}' is already up to date.", name)),
    }
}

/// Link or unlink a vault credential to a skill.
pub async fn exec_gw_skill_link_secret(
    args: &serde_json::Value,
    skill_mgr: &SharedSkillManager,
) -> Result<String, String> {
    let action = args
        .get("action")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: action".to_string())?;
    let skill = args
        .get("skill")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: skill".to_string())?;
    let secret = args
        .get("secret")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: secret".to_string())?;

    let mut mgr = skill_mgr.lock().await;
    match action {
        "link" => {
            mgr.link_secret(skill, secret).map_err(|e| e.to_string())?;
            Ok(format!(
                "Secret '{}' linked to skill '{}'.",
                secret, skill,
            ))
        }
        "unlink" => {
            mgr.unlink_secret(skill, secret).map_err(|e| e.to_string())?;
            Ok(format!(
                "Secret '{}' unlinked from skill '{}'.",
                secret, skill,
            ))
        }
        _ => Err(format!(
            "Unknown action '{}'. Use 'link' or 'unlink'.",
            action,
        )),
    }
}
