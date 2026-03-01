//! Data source abstraction.

use xjasper_core::types::Field;

pub trait DataSource {
    fn next(&mut self) -> Result<bool, DataSourceError>;
    fn get_field(&self, name: &str) -> Result<String, DataSourceError>;
    fn reset(&mut self) -> Result<(), DataSourceError>;
}

#[derive(Debug, thiserror::Error)]
pub enum DataSourceError {
    #[error("Field not found: {0}")]
    FieldNotFound(String),
    #[error("Data source error: {0}")]
    Other(String),
}

pub struct JsonDataSource;
