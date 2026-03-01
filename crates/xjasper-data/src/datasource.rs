//! Data source abstraction.

use serde_json::Value;
use std::collections::HashMap;

pub trait DataSource {
    fn next(&mut self) -> Result<bool, DataSourceError>;
    fn get_field(&self, name: &str) -> Result<String, DataSourceError>;
    fn reset(&mut self) -> Result<(), DataSourceError>;
}

#[derive(Debug, thiserror::Error)]
pub enum DataSourceError {
    #[error("Field not found: {0}")]
    FieldNotFound(String),

    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Data source error: {0}")]
    Other(String),
}

/// JSON array data source
pub struct JsonDataSource {
    data: Vec<HashMap<String, Value>>,
    current_index: Option<usize>,
}

impl JsonDataSource {
    /// Create a new JSON data source from a JSON array string
    pub fn new(json: &str) -> Result<Self, DataSourceError> {
        let data: Vec<HashMap<String, Value>> = serde_json::from_str(json)?;
        Ok(Self {
            data,
            current_index: None,
        })
    }

    /// Get current row data
    fn current_row(&self) -> Result<&HashMap<String, Value>, DataSourceError> {
        match self.current_index {
            Some(idx) => self.data.get(idx).ok_or_else(|| {
                DataSourceError::Other("Invalid current index".to_string())
            }),
            None => Err(DataSourceError::Other("No current row".to_string())),
        }
    }
}

impl DataSource for JsonDataSource {
    fn next(&mut self) -> Result<bool, DataSourceError> {
        match self.current_index {
            None => {
                // First call
                if self.data.is_empty() {
                    Ok(false)
                } else {
                    self.current_index = Some(0);
                    Ok(true)
                }
            }
            Some(idx) => {
                let next_idx = idx + 1;
                if next_idx < self.data.len() {
                    self.current_index = Some(next_idx);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn get_field(&self, name: &str) -> Result<String, DataSourceError> {
        let row = self.current_row()?;
        let value = row.get(name).ok_or_else(|| {
            DataSourceError::FieldNotFound(name.to_string())
        })?;

        // Convert JSON value to string
        let result = match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => String::new(),
            _ => value.to_string(),
        };

        Ok(result)
    }

    fn reset(&mut self) -> Result<(), DataSourceError> {
        self.current_index = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_datasource() {
        let json = r#"[
            {"name": "Alice", "amount": "100.50"},
            {"name": "Bob", "amount": "200.75"}
        ]"#;

        let mut ds = JsonDataSource::new(json).unwrap();

        // First row
        assert!(ds.next().unwrap());
        assert_eq!(ds.get_field("name").unwrap(), "Alice");
        assert_eq!(ds.get_field("amount").unwrap(), "100.50");

        // Second row
        assert!(ds.next().unwrap());
        assert_eq!(ds.get_field("name").unwrap(), "Bob");

        // No more rows
        assert!(!ds.next().unwrap());
    }

    #[test]
    fn test_reset() {
        let json = r#"[{"name": "Alice"}]"#;
        let mut ds = JsonDataSource::new(json).unwrap();

        assert!(ds.next().unwrap());
        assert_eq!(ds.get_field("name").unwrap(), "Alice");

        ds.reset().unwrap();

        assert!(ds.next().unwrap());
        assert_eq!(ds.get_field("name").unwrap(), "Alice");
    }
}
