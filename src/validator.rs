pub fn validate_config(yaml_config: &yaml_rust::Yaml) -> Result<(), Box<dyn std::error::Error>> {
    let required_fields = vec![
        ("server.host", &["server", "host"], "string"),
        ("server.port", &["server", "port"], "integer"),
        ("worker.id", &["worker", "id"], "string"),
        ("worker.pk", &["worker", "pk"], "string"),
        ("worker.schedule", &["worker", "schedule"], "string"),
        ("worker.modules", &["worker", "modules"], "array"),
    ];

    for (field_name, path, expected_type) in required_fields {
        let mut current = yaml_config;

        for &key in path {
            current = &current[key];
            if current.is_badvalue() {
                return Err(format!("Missing required configuration field: {}", field_name).into());
            }
        }

        let type_valid = match expected_type {
            "string" => current.as_str().is_some(),
            "integer" => current.as_i64().is_some(),
            "array" => current.is_array(),
            _ => true,
        };

        if !type_valid {
            return Err(format!(
                "Invalid type for {}: expected {}",
                field_name, expected_type
            )
            .into());
        }
    }

    Ok(())
}
