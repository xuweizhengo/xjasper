//! XJasper Engine
//!
//! Facade API for the report engine.

use xjasper_core::Template;
use xjasper_data::JsonDataSource;
use xjasper_layout::LayoutEngine;
use xjasper_render::PdfRenderer;

pub struct ReportEngine;

impl ReportEngine {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ReportEngine {
    fn default() -> Self {
        Self::new()
    }
}
