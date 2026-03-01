//! XJasper Engine
//!
//! Facade API for the report engine.

use xjasper_core::template::parse_template;
use xjasper_data::datasource::JsonDataSource;
use xjasper_layout::LayoutEngine;
use xjasper_render::PdfRenderer;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Template error: {0}")]
    TemplateError(String),

    #[error("Data error: {0}")]
    DataError(String),

    #[error("Layout error: {0}")]
    LayoutError(String),

    #[error("Render error: {0}")]
    RenderError(String),
}

pub type Result<T> = std::result::Result<T, EngineError>;

pub struct ReportEngine {
    layout_engine: LayoutEngine,
    pdf_renderer: PdfRenderer,
}

impl ReportEngine {
    pub fn new() -> Self {
        Self {
            layout_engine: LayoutEngine::new(),
            pdf_renderer: PdfRenderer::new(),
        }
    }

    /// Render a report from JSON template and data to PDF bytes
    pub fn render(&mut self, template_json: &str, data_json: &str) -> Result<Vec<u8>> {
        // Parse template
        let template = parse_template(template_json)
            .map_err(|e| EngineError::TemplateError(e.to_string()))?;

        // Create data source
        let mut data_source = JsonDataSource::new(data_json)
            .map_err(|e| EngineError::DataError(e.to_string()))?;

        // Layout
        let filled_doc = self
            .layout_engine
            .layout(&template, &mut data_source)
            .map_err(|e| EngineError::LayoutError(e.to_string()))?;

        // Render to PDF
        let pdf_bytes = self
            .pdf_renderer
            .render(&filled_doc)
            .map_err(|e| EngineError::RenderError(e.to_string()))?;

        Ok(pdf_bytes)
    }
}

impl Default for ReportEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple_report() {
        let template = r#"{
            "name": "test",
            "version": "0.1",
            "page": {
                "width": 595,
                "height": 842,
                "margins": [40, 40, 40, 40]
            },
            "fields": [
                {"name": "name", "type": "string"}
            ],
            "variables": [],
            "bands": {
                "title": {
                    "height": 60,
                    "elements": [
                        {
                            "type": "staticText",
                            "x": 0,
                            "y": 10,
                            "width": 515,
                            "height": 40,
                            "text": "Test Report"
                        }
                    ]
                },
                "detail": {
                    "height": 20,
                    "elements": [
                        {
                            "type": "textField",
                            "x": 0,
                            "y": 0,
                            "width": 300,
                            "height": 20,
                            "expression": "$F{name}"
                        }
                    ]
                }
            }
        }"#;

        let data = r#"[
            {"name": "Alice"},
            {"name": "Bob"}
        ]"#;

        let mut engine = ReportEngine::new();
        let result = engine.render(template, data);

        assert!(result.is_ok());
        let pdf_bytes = result.unwrap();
        assert!(!pdf_bytes.is_empty());
        assert!(pdf_bytes.starts_with(b"%PDF"));
    }
}

