//! Template parser.

use crate::types::Template;
use serde_json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TemplateError {
    #[error("Failed to parse JSON: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid template: {0}")]
    ValidationError(String),
}

pub type Result<T> = std::result::Result<T, TemplateError>;

/// Parse a JSON template string into a Template struct
pub fn parse_template(json: &str) -> Result<Template> {
    let template: Template = serde_json::from_str(json)?;
    validate_template(&template)?;
    Ok(template)
}

/// Validate template structure
fn validate_template(template: &Template) -> Result<()> {
    // Check version
    if template.version != "0.1" {
        return Err(TemplateError::ValidationError(
            format!("Unsupported version: {}", template.version)
        ));
    }

    // Check page dimensions
    if template.page.width == 0 || template.page.height == 0 {
        return Err(TemplateError::ValidationError(
            "Page width and height must be greater than 0".to_string()
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_template() {
        let json = r#"{
            "name": "test",
            "version": "0.1",
            "page": {
                "width": 595,
                "height": 842,
                "margins": [40, 40, 40, 40]
            },
            "fields": [],
            "variables": [],
            "bands": {}
        }"#;

        let result = parse_template(json);
        assert!(result.is_ok());

        let template = result.unwrap();
        assert_eq!(template.name, "test");
        assert_eq!(template.version, "0.1");
        assert_eq!(template.page.width, 595);
    }

    #[test]
    fn test_invalid_version() {
        let json = r#"{
            "name": "test",
            "version": "999.0",
            "page": {
                "width": 595,
                "height": 842,
                "margins": [40, 40, 40, 40]
            },
            "fields": [],
            "variables": [],
            "bands": {}
        }"#;

        let result = parse_template(json);
        assert!(result.is_err());
    }
}
