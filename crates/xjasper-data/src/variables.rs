//! Variable aggregation.

use rust_decimal::Decimal;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VariableError {
    #[error("Invalid decimal value: {0}")]
    DecimalError(String),

    #[error("Variable error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, VariableError>;

/// Variable aggregation calculator
#[derive(Debug, Clone)]
pub struct VariableCalculator {
    name: String,
    calculation_type: CalculationType,
    value: Decimal,
    count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CalculationType {
    Sum,
    Count,
    Average,
    Min,
    Max,
    First,
}

impl VariableCalculator {
    pub fn new(name: String, calculation_type: CalculationType) -> Self {
        Self {
            name,
            calculation_type,
            value: Decimal::ZERO,
            count: 0,
        }
    }

    /// Update the variable with a new value
    pub fn update(&mut self, value_str: &str) -> Result<()> {
        let value = Decimal::from_str(value_str).map_err(|e| {
            VariableError::DecimalError(format!("{}: {}", value_str, e))
        })?;

        match self.calculation_type {
            CalculationType::Sum => {
                self.value += value;
            }
            CalculationType::Count => {
                self.value = Decimal::from(self.count + 1);
            }
            CalculationType::Average => {
                self.value += value;
            }
            CalculationType::Min => {
                if self.count == 0 || value < self.value {
                    self.value = value;
                }
            }
            CalculationType::Max => {
                if self.count == 0 || value > self.value {
                    self.value = value;
                }
            }
            CalculationType::First => {
                if self.count == 0 {
                    self.value = value;
                }
            }
        }

        self.count += 1;
        Ok(())
    }

    /// Get the current calculated value
    pub fn get_value(&self) -> String {
        match self.calculation_type {
            CalculationType::Average => {
                if self.count > 0 {
                    let avg = self.value / Decimal::from(self.count);
                    avg.to_string()
                } else {
                    "0".to_string()
                }
            }
            _ => self.value.to_string(),
        }
    }

    /// Reset the variable
    pub fn reset(&mut self) {
        self.value = Decimal::ZERO;
        self.count = 0;
    }
}

impl FromStr for CalculationType {
    type Err = VariableError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "Sum" => Ok(CalculationType::Sum),
            "Count" => Ok(CalculationType::Count),
            "Average" => Ok(CalculationType::Average),
            "Min" => Ok(CalculationType::Min),
            "Max" => Ok(CalculationType::Max),
            "First" => Ok(CalculationType::First),
            _ => Err(VariableError::Other(format!("Unknown calculation type: {}", s))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum() {
        let mut calc = VariableCalculator::new("total".to_string(), CalculationType::Sum);

        calc.update("100.50").unwrap();
        calc.update("200.75").unwrap();
        calc.update("150.25").unwrap();

        assert_eq!(calc.get_value(), "451.50");
    }

    #[test]
    fn test_count() {
        let mut calc = VariableCalculator::new("count".to_string(), CalculationType::Count);

        calc.update("100").unwrap();
        calc.update("200").unwrap();

        assert_eq!(calc.get_value(), "2");
    }

    #[test]
    fn test_average() {
        let mut calc = VariableCalculator::new("avg".to_string(), CalculationType::Average);

        calc.update("100").unwrap();
        calc.update("200").unwrap();
        calc.update("300").unwrap();

        assert_eq!(calc.get_value(), "200");
    }
}
