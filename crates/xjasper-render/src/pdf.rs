//! PDF renderer.

use xjasper_layout::filled::{FilledDocument, FilledElement};
use printpdf::*;
use thiserror::Error;
use std::io::BufWriter;

#[derive(Debug, Error)]
pub enum PdfError {
    #[error("PDF generation error: {0}")]
    GenerationError(String),
}

pub type Result<T> = std::result::Result<T, PdfError>;

pub struct PdfRenderer;

impl PdfRenderer {
    pub fn new() -> Self {
        Self
    }

    /// Render a filled document to PDF bytes
    pub fn render(&self, doc: &FilledDocument) -> Result<Vec<u8>> {
        if doc.pages.is_empty() {
            return Err(PdfError::GenerationError("No pages to render".to_string()));
        }

        let first_page = &doc.pages[0];
        let (pdf_doc, page_idx, layer_idx) = PdfDocument::new(
            "XJasper Report",
            Mm(first_page.width as f32 * 0.352778), // points to mm
            Mm(first_page.height as f32 * 0.352778),
            "Layer 1",
        );

        // Add built-in font
        let font = pdf_doc
            .add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| PdfError::GenerationError(format!("Font error: {:?}", e)))?;

        let current_layer = pdf_doc.get_page(page_idx).get_layer(layer_idx);

        // Render elements
        for element in &first_page.elements {
            self.render_element(&current_layer, element, first_page.height, &font)?;
        }

        // Save to bytes
        let mut buffer = Vec::new();
        {
            let mut buf_writer = BufWriter::new(&mut buffer);
            pdf_doc
                .save(&mut buf_writer)
                .map_err(|e| PdfError::GenerationError(format!("Failed to save PDF: {:?}", e)))?;
        } // buf_writer dropped here

        Ok(buffer)
    }

    fn render_element(
        &self,
        layer: &PdfLayerReference,
        element: &FilledElement,
        page_height: u32,
        font: &IndirectFontRef,
    ) -> Result<()> {
        match element {
            FilledElement::Text(text) => {
                // Convert coordinates (PDF origin is bottom-left)
                let x_mm = Mm(text.x as f32 * 0.352778);
                let y_mm = Mm((page_height - text.y - text.height) as f32 * 0.352778);

                // Get font size
                let font_size = text.style.font_size.unwrap_or(12) as f32;

                // Write text
                layer.use_text(&text.text, font_size, x_mm, y_mm, font);

                Ok(())
            }
        }
    }
}

impl Default for PdfRenderer {
    fn default() -> Self {
        Self::new()
    }
}


