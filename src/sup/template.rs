use regex::Regex;
use std::collections::{HashMap, HashSet};
use toml::{value::Table, Value};

/// Wrapper struct for the expanded TOML output
pub(crate) struct Expanded(pub(crate) String);

/// Replace all {{ var_name }} occurrences with corresponding values from the map
pub(crate) fn render_template_string(template: &str, vars: &HashMap<String, String>) -> String {
    let re = Regex::new(r"\{\{\s*([a-zA-Z_-][a-zA-Z0-9_-]*)\s*\}\}").unwrap();

    re.replace_all(template, |caps: &regex::Captures| {
        let key = &caps[1];
        vars.get(key).cloned().unwrap_or_default()
    })
    .into_owned()
}

/// Validate that all required variables in the template are provided, and no extra ones are present
pub(crate) fn check_vars_in_string(
    template_str: &str,
    input_map: &HashMap<String, String>,
) -> Result<(), String> {
    let required_vars = extract_vars_from_template(template_str);

    // Extra variables not used in the template
    let extra_vars: Vec<String> = input_map
        .keys()
        .filter(|key| *key != "template" && !required_vars.contains(*key))
        .cloned()
        .collect();

    if !extra_vars.is_empty() {
        return Err(format!("Extra variables in input map: {:?}", extra_vars));
    }

    // Missing variables expected in the template
    let missing_vars: Vec<String> = required_vars
        .iter()
        .filter(|var| !input_map.contains_key(*var))
        .cloned()
        .collect();

    if !missing_vars.is_empty() {
        return Err(format!(
            "Missing variables in input map: {:?}",
            missing_vars
        ));
    }

    Ok(())
}

/// Extract all variable names of the form {{ var_name }} from the template string
pub(crate) fn extract_vars_from_template(template_str: &str) -> HashSet<String> {
    let re = Regex::new(r"\{\{\s*([a-zA-Z_-][a-zA-Z0-9_-]*)\s*\}\}").unwrap();
    re.captures_iter(template_str)
        .filter_map(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .collect()
}
