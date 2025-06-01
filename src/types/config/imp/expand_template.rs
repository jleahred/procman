use crate::sup::template::{check_vars_in_string, render_template_string};
use std::collections::HashMap;
use toml::{value::Table, Value};

/// Wrapper struct for the expanded TOML output
pub(crate) struct Expanded(pub(crate) String);

/// Main function to expand templates in the TOML input using a map of templates
pub(super) fn expand_template(
    input: &str,
    templates: &HashMap<String, String>,
) -> Result<Expanded, String> {
    // Parse the TOML input
    let mut data: Value = toml::from_str(&input).map_err(|e| e.to_string())?;

    // Access the "process" array
    let processes = data
        .get_mut("process")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "No processes found".to_string())?;

    // Iterate over each process
    for proc in processes {
        let id = proc
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "Process ID not found or is not a string".to_string())?
            .to_string();

        // If there's a [template] block, process it
        if let Some(template_table) = proc
            .get_mut("template")
            .and_then(Value::as_table_mut)
            .map(|t| t.clone())
        {
            // Convert the template section into a HashMap<String, String>
            let input_map: HashMap<String, String> = template_table
                .iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                .collect();

            // Get the template name from the input map
            let template_name = input_map
                .get("template")
                .ok_or_else(|| format!("Missing 'template' field in process '{}'", id))?;

            // Fetch the actual template string
            let template_str = templates
                .get(template_name)
                .ok_or_else(|| format!("Template '{}' not found", template_name))?;

            // Ensure that all required variables are present and no extras exist
            check_vars_in_string(template_str, &input_map).map_err(|e| {
                format!(
                    "Error in template '{}', process id '{}': {}",
                    template_name, id, e
                )
            })?;

            // Render the template manually
            let rendered_str = render_template_string(template_str, &input_map);

            // Parse the rendered result as TOML
            let rendered_map: Table = toml::from_str(&rendered_str)
                .map_err(|_| format!("Template does not produce valid TOML\n{}", &rendered_str))?;

            // Merge the rendered fields into the original process table
            proc.as_table_mut()
                .unwrap()
                .extend(rendered_map.into_iter());
            proc.as_table_mut().unwrap().remove("template");
        }
    }

    // Remove the top-level "template" key if it exists
    if let Some(_) = data.get_mut("template").and_then(Value::as_array_mut) {
        data.as_table_mut().unwrap().remove("template");
    }

    // Serialize the modified TOML structure back to a string
    let output = toml::to_string_pretty(&data).unwrap();
    Ok(Expanded(output))
}
