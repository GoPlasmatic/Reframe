# Workflows Organization

This directory contains the workflow definitions for the Swift MT to MX transformation engine (CBPR+). The workflows are organized by message type for better maintainability.

## Structure

### Root Level
- `parse.json` - Common MT message parser (handles all message types)
- `index.json` - Defines the loading order of all workflows

### Message Type Subfolders

#### `MT103/` - Normal and STP Credit Transfer Messages
- `bah-mapping.json` - Business Application Header mapping
- `precondition.json` - Precondition checks 
- `document-mapping.json` - Document mapping to pacs.008
- `combine-cbpr.json` - XML combiner

#### `MT103REJT/` - Rejection Messages
- `bah-mapping.json` - Business Application Header mapping for rejections
- `precondition.json` - Precondition checks for rejections
- `document-mapping.json` - Document mapping to pacs.002
- `combine-cbpr.json` - XML combiner for rejections

#### `MT103RETN/` - Return Messages
- `bah-mapping.json` - Business Application Header mapping for returns
- `precondition.json` - Precondition checks for returns
- `document-mapping.json` - Document mapping to pacs.004
- `combine-cbpr.json` - XML combiner for returns

## Workflow Loading

The workflows are loaded in the order specified in `index.json`. Each workflow defines:
- `id` - Unique identifier for the workflow
- `path` - Relative path to the workflow file
- `description` - Human-readable description

## Workflow Dependencies

Workflows reference each other through the `progress.workflow_id` condition field:
- MT103 workflows: `parser` → `mt103-bah-mapper` → `mt103-precondition` → `mt103-document-mapper` → `mt103-combine-cbpr`
- MT103REJT workflows: `parser` → `mt103-rejt-bah-mapper` → `mt103-rejt-precondition` → `mt103-rejt-document-mapper` → `mt103-rejt-combine-cbpr`
- MT103RETN workflows: `parser` → `mt103-retn-bah-mapper` → `mt103-retn-precondition` → `mt103-retn-document-mapper` → `mt103-retn-combine-cbpr`

## Benefits of This Organization

1. **Better Maintainability** - Related workflows are grouped together
2. **Clear Dependencies** - Easy to understand workflow relationships 
3. **Scalable** - New message types can be added as new subfolders
4. **Consistent Naming** - No more numbered prefixes, semantic names instead
5. **Self-Documenting** - Structure reflects the business logic organization

## JSON Semi-Beautifier

A smart JSON formatter tool (`json_semi_beautifier.py`) is available in the root directory that strikes a balance between minification and beautification, specifically optimized for JSONLogic workflow files.

### Features

- **Smart Formatting**: Keeps simple structures compact while breaking down complex ones
- **JSONLogic Aware**: Recognizes common JSONLogic operators (`var`, `==`, `if`, `and`, `or`, etc.)
- **Configurable**: Adjust inline length, item counts, and indentation
- **Batch Processing**: Format multiple files at once
- **In-Place Editing**: Option to modify files directly

### Usage

Format and display a file:
```bash
../json_semi_beautifier.py forward/MT103/bah-mapping.json
```

Format multiple files in place:
```bash
../json_semi_beautifier.py --in-place forward/MT103/*.json
```

### Options

- `--in-place, -i`: Format files in place (modifies the original files)
- `--max-inline-length N`: Maximum length for inline objects/arrays (default: 80)
- `--max-inline-items N`: Maximum number of items for inline objects/arrays (default: 3)
- `--indent N`: Number of spaces for indentation (default: 4)

### Example

Before (Standard JSON formatting):
```json
{
  "condition": {
    "and": [
      {
        "==": [
          {
            "var": "SwiftMT.message_type"
          },
          "103"
        ]
      }
    ]
  }
}
```

After (Semi-beautified):
```json
{
    "condition": { "and": [
        {"==": [{"var": "SwiftMT.message_type"}, "103"]}
    ]}
}
``` 