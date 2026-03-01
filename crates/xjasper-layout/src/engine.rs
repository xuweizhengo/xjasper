//! Layout engine.

use crate::filled::{FilledDocument, FilledElement, FilledPage, FilledText};
use xjasper_core::expression::{parse_expression, Expression};
use xjasper_core::types::{Element, Template};
use xjasper_data::datasource::DataSource;
use xjasper_data::variables::{CalculationType, VariableCalculator};
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LayoutError {
    #[error("Data source error: {0}")]
    DataSourceError(String),

    #[error("Expression error: {0}")]
    ExpressionError(String),

    #[error("Variable error: {0}")]
    VariableError(String),

    #[error("Layout error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, LayoutError>;

pub struct LayoutEngine {
    variables: HashMap<String, VariableCalculator>,
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// Layout a template with data
    pub fn layout(
        &mut self,
        template: &Template,
        data_source: &mut dyn DataSource,
    ) -> Result<FilledDocument> {
        // Initialize variables
        self.init_variables(template)?;

        let mut doc = FilledDocument::new();
        let mut page = FilledPage::new(template.page.width, template.page.height);

        let mut current_y = template.page.margins[0]; // top margin

        // Render title band
        if let Some(title_band) = template.bands.get("title") {
            for element in &title_band.elements {
                let filled = self.fill_element(element, data_source, current_y)?;
                page.add_element(filled);
            }
            current_y += title_band.height;
        }

        // Render detail band for each row
        if let Some(detail_band) = template.bands.get("detail") {
            while data_source.next().map_err(|e| LayoutError::DataSourceError(e.to_string()))? {
                // Update variables
                self.update_variables(template, data_source)?;

                // Render detail elements
                for element in &detail_band.elements {
                    let filled = self.fill_element(element, data_source, current_y)?;
                    page.add_element(filled);
                }
                current_y += detail_band.height;
            }
        }

        // Render summary band
        if let Some(summary_band) = template.bands.get("summary") {
            for element in &summary_band.elements {
                let filled = self.fill_element(element, data_source, current_y)?;
                page.add_element(filled);
            }
        }

        doc.add_page(page);
        Ok(doc)
    }

    fn init_variables(&mut self, template: &Template) -> Result<()> {
        for var in &template.variables {
            let calc_type = CalculationType::from_str(&var.calculation)
                .map_err(|e| LayoutError::VariableError(e.to_string()))?;
            let calculator = VariableCalculator::new(var.name.clone(), calc_type);
            self.variables.insert(var.name.clone(), calculator);
        }
        Ok(())
    }

    fn update_variables(
        &mut self,
        template: &Template,
        data_source: &dyn DataSource,
    ) -> Result<()> {
        for var in &template.variables {
            let expr = parse_expression(&var.expression)
                .map_err(|e| LayoutError::ExpressionError(e.to_string()))?;

            let value = self.evaluate_expression(&expr, data_source)?;

            if let Some(calculator) = self.variables.get_mut(&var.name) {
                calculator
                    .update(&value)
                    .map_err(|e| LayoutError::VariableError(e.to_string()))?;
            }
        }
        Ok(())
    }

    fn fill_element(
        &self,
        element: &Element,
        data_source: &dyn DataSource,
        y_offset: u32,
    ) -> Result<FilledElement> {
        match element {
            Element::StaticText(st) => {
                let filled = FilledText {
                    x: st.x,
                    y: y_offset + st.y,
                    width: st.width,
                    height: st.height,
                    text: st.text.clone(),
                    style: st.style.clone(),
                };
                Ok(FilledElement::Text(filled))
            }
            Element::TextField(tf) => {
                let expr = parse_expression(&tf.expression)
                    .map_err(|e| LayoutError::ExpressionError(e.to_string()))?;

                let text = self.evaluate_expression(&expr, data_source)?;

                let filled = FilledText {
                    x: tf.x,
                    y: y_offset + tf.y,
                    width: tf.width,
                    height: tf.height,
                    text,
                    style: tf.style.clone(),
                };
                Ok(FilledElement::Text(filled))
            }
        }
    }

    fn evaluate_expression(
        &self,
        expr: &Expression,
        data_source: &dyn DataSource,
    ) -> Result<String> {
        match expr {
            Expression::FieldRef(field_name) => data_source
                .get_field(field_name)
                .map_err(|e| LayoutError::DataSourceError(e.to_string())),
            Expression::VariableRef(var_name) => {
                if let Some(calculator) = self.variables.get(var_name) {
                    Ok(calculator.get_value())
                } else {
                    Err(LayoutError::VariableError(format!(
                        "Variable not found: {}",
                        var_name
                    )))
                }
            }
            Expression::Literal(text) => Ok(text.clone()),
        }
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

