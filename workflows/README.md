# Workflows Directory

This directory contains JSON workflow definitions that are automatically loaded by the Reframe API at startup.

## How It Works

When the Reframe API starts up, it:
1. Scans this directory for `.json` files
2. Loads each valid workflow file into the dataflow engine
3. Reports the number of successfully loaded workflows

## Adding New Workflows

To add a new workflow:

1. Create a new `.json` file in this directory (e.g., `my-custom-workflow.json`)
2. Define your workflow using the JSON schema (see examples below)
3. Restart the application or redeploy

## Workflow JSON Schema

A workflow must have the following structure:

```json
{
    "id": "unique_workflow_id",
    "name": "Human Readable Workflow Name",
    "tasks": [
        {
            "id": "task_id",
            "name": "Task Name",
            "function": {
                "name": "function_name",
                "input": {
                    // Function-specific parameters
                }
            }
        }
    ]
}
```

## Available Functions

### Parse Function
Parses input messages into structured data:

```json
{
    "name": "parse",
    "input": {
        "format": "SwiftMT",
        "input_field_name": "payload",
        "output_field_name": "SwiftMT"
    }
}
```

### Map Function
Maps data from source to target using JsonLogic:

```json
{
    "name": "map",
    "input": {
        "mappings": [
            {
                "path": "data.target.field",
                "logic": { "var": "data.source.field" }
            }
        ]
    }
}
```

### Publish Function
Formats and outputs the final result:

```json
{
    "name": "publish",
    "input": {
        "input_field_name": "MX",
        "output_format": "pacs.008.001.13"
    }
}
```

## Example Workflow

The `mt103-pacs008-mapping.json` file contains a complete example that transforms SWIFT MT103 messages to ISO 20022 pacs.008.001.13 format.

## JsonLogic Mapping

The mapping function uses JsonLogic syntax for transformations. Common patterns:

### Simple Field Mapping
```json
{ "var": "data.source.field" }
```

### Conditional Mapping
```json
{
    "if": [
        { "var": "data.condition" },
        "value_if_true",
        "value_if_false"
    ]
}
```

### Comparison Operations
```json
{
    "if": [
        { "==": [{ "var": "data.field" }, "expected_value"] },
        "result_if_equal",
        "result_if_not_equal"
    ]
}
```

## Development Tips

1. **Validate JSON**: Ensure your workflow files are valid JSON before deploying
2. **Test Incrementally**: Start with simple workflows and add complexity gradually
3. **Use Descriptive Names**: Use clear task and workflow names for easier debugging
4. **Check Logs**: Monitor the application logs for workflow loading status

## Error Handling

If a workflow file fails to load:
- Check the application logs for specific error messages
- Validate your JSON syntax
- Ensure all required fields are present
- Verify function names and parameters

The application will continue to run even if some workflows fail to load, but you'll see error messages in the logs.

## Redeployment

After adding or modifying workflows:

1. **Local Development**: Restart the application
2. **Production**: Redeploy using the automated CI/CD pipeline
3. **Docker**: Rebuild and restart the container

The workflows are embedded in the Docker image, so any changes require a rebuild and redeployment. 