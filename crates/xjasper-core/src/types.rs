//! Core types for XJasper templates.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub version: String,
    pub page: PageConfig,
    pub fields: Vec<Field>,
    pub variables: Vec<Variable>,
    pub bands: BandMap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageConfig {
    pub width: u32,
    pub height: u32,
    pub margins: [u32; 4], // [top, right, bottom, left]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    #[serde(rename = "type")]
    pub var_type: String,
    pub calculation: String,
    pub expression: String,
}

pub type BandMap = HashMap<String, Band>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Band {
    pub height: u32,
    pub elements: Vec<Element>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Element {
    #[serde(rename = "staticText")]
    StaticText(StaticText),
    #[serde(rename = "textField")]
    TextField(TextField),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticText {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub text: String,
    #[serde(default)]
    pub style: TextStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextField {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub expression: String,
    #[serde(default)]
    pub style: TextStyle,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextStyle {
    #[serde(rename = "fontSize")]
    pub font_size: Option<u32>,
    #[serde(rename = "fontWeight")]
    pub font_weight: Option<String>,
    pub align: Option<String>,
}
