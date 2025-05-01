use minijinja::{Environment, Value as JinjaValue};
use std::collections::{HashMap, HashSet};
use toml::{value::Table, Value};

pub(crate) struct Expanded(pub(crate) String);

pub(super) fn expand_template(
    input: &str,
    templates: &HashMap<String, String>,
) -> Result<Expanded, String> {
    let mut data: Value = toml::from_str(&input).map_err(|e| e.to_string())?;

    // Access processes
    let processes = data
        .get_mut("process")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "No processes found".to_string())?;

    let mut jinja_env = Environment::new();

    for (name, template_str) in templates {
        jinja_env
            .add_template(name, template_str)
            .map_err(|_| format!("Invalid template: {}", name))?;
    }

    for proc in processes {
        let id = proc
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "Process ID not found or is not a string".to_string())?
            .to_string();

        // Clone template table as a map to pass it to the template engine
        if let Some(template_table) = proc
            .get_mut("template")
            .and_then(Value::as_table_mut)
            .map(|t| t.clone())
        {
            let input_map: HashMap<String, String> = template_table
                .iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                .collect();

            // Renderizar usando minijinja
            let template_name = input_map
                .get("template")
                .map(String::as_str)
                .ok_or_else(|| {
                    format!(
                        "Procesing template for process.id: <{}>. 'template' not found in: {:#?}",
                        id, input_map
                    )
                })?;

            let jinja_template = jinja_env
                .get_template(template_name)
                .map_err(|e| e.to_string())?;

            check_vars(&jinja_template, &input_map).map_err(|e| {
                format!(
                    "Error in template '{}', process id '{}': {}",
                    template_name, id, e
                )
            })?;

            let rendered_str = jinja_template
                .render(JinjaValue::from(input_map.clone()))
                .map_err(|_| "Error rendering template".to_string())?;

            // Parsear el resultado como TOML
            let rendered_map: Table = toml::from_str(&rendered_str)
                .map_err(|_| format!("Template does not produce valid TOML\n{}", &rendered_str))?;

            // Reemplazar process.template
            proc.as_table_mut()
                .unwrap()
                .extend(rendered_map.into_iter());
            proc.as_table_mut().unwrap().remove("template");
        }
    }

    // Remove the "template" section from the data
    if let Some(_templates) = data.get_mut("template").and_then(Value::as_array_mut) {
        data.as_table_mut().unwrap().remove("template");
    }

    let output = toml::to_string_pretty(&data).unwrap();
    Ok(Expanded(output))
}

use regex::Regex;

fn extract_vars_from_template(template_str: &str) -> HashSet<String> {
    let re = Regex::new(r"\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\}\}").unwrap();
    re.captures_iter(template_str)
        .filter_map(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .collect()
}

fn check_vars(
    template: &minijinja::Template,
    input_map: &HashMap<String, String>,
) -> Result<(), String> {
    let template_str = template.source();

    // Check for extra variables in the input map
    let required_vars = extract_vars_from_template(template_str);

    let extra_vars: Vec<String> = input_map
        .keys()
        .filter(|key| *key != "template" && !required_vars.contains(*key))
        .cloned()
        .collect();

    if !extra_vars.is_empty() {
        return Err(format!("Extra variables in input map: {:?}", extra_vars));
    }

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
