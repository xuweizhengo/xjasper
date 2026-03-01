# XJasper Examples

This directory contains example templates and data for XJasper.

## Files

- `simple-invoice.json` - A minimal invoice template demonstrating basic features
- `data.json` - Sample data for the invoice template

## Template Structure

The `simple-invoice.json` template demonstrates:

- **3 Bands**: title, detail, summary
- **2 Element Types**: staticText, textField
- **Field References**: `$F{customerName}`, `$F{amount}`
- **Variable Aggregation**: `$V{total}` (Sum of amounts)
- **Text Styling**: fontSize, fontWeight, align

## Expected Output

When rendered with `data.json`, the PDF should contain:

```
Invoice
-------

Alice Johnson                                100.50
Bob Smith                                    200.75
Charlie Brown                                150.25

                    Total:                   451.50
```

## Usage (Stage 1+)

Once the CLI is implemented:

```bash
cargo run --bin xjasper -- \
  --template examples/simple-invoice.json \
  --data examples/data.json \
  --output output.pdf
```

## Schema Validation

Validate templates against the JSON Schema:

```bash
# Using ajv-cli
npm install -g ajv-cli
ajv validate -s schema/template-v0.1.json -d examples/simple-invoice.json
```
