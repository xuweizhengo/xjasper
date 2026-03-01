//! Filled document structure.

use xjasper_core::types::TextStyle;

/// A filled document ready for rendering
#[derive(Debug, Clone)]
pub struct FilledDocument {
    pub pages: Vec<FilledPage>,
}

#[derive(Debug, Clone)]
pub struct FilledPage {
    pub width: u32,
    pub height: u32,
    pub elements: Vec<FilledElement>,
}

#[derive(Debug, Clone)]
pub enum FilledElement {
    Text(FilledText),
}

#[derive(Debug, Clone)]
pub struct FilledText {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub text: String,
    pub style: TextStyle,
}

impl FilledDocument {
    pub fn new() -> Self {
        Self { pages: Vec::new() }
    }

    pub fn add_page(&mut self, page: FilledPage) {
        self.pages.push(page);
    }
}

impl Default for FilledDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl FilledPage {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            elements: Vec::new(),
        }
    }

    pub fn add_element(&mut self, element: FilledElement) {
        self.elements.push(element);
    }
}
