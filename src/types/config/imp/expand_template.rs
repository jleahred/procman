use minijinja::{Environment, Value as JinjaValue};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};
use toml::{value::Table, Value};

pub(crate) struct Expanded(pub(crate) String);

pub(super) fn expand_template(input: &str, template_str: &str) -> Result<Expanded, String> {
    let mut data: Value = toml::from_str(&input).map_err(|e| e.to_string())?;

    // Access processes
    let processes = data
        .get_mut("process")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "No processes found".to_string())?;

    let mut jinja_env = Environment::new();

    jinja_env
        .add_template("PROCESS PODMAN", &template_str)
        .map_err(|_| "Invalid template".to_string())?;

    for proc in processes {
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
                .get("name")
                .map(String::as_str)
                .unwrap_or("PROCESS PODMAN");

            let rendered_str = jinja_env
                .get_template(template_name)
                .expect("Template not found")
                .render(JinjaValue::from(input_map.clone()))
                .expect("Error rendering template");

            // Parsear el resultado como TOML
            let rendered_map: Table = toml::from_str(&rendered_str)
                .map_err(|_| format!("Template does not produce valid TOML\n{}", &rendered_str))?;

            // Reemplazar process.template
            proc.as_table_mut()
                .unwrap()
                .insert("template".to_string(), Value::Table(rendered_map));
        }
    }

    let output = toml::to_string_pretty(&data).unwrap();
    Ok(Expanded(output))
}
