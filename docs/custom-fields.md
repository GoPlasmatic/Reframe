# Custom Fields

## Overview

Custom fields allow packages to define computed fields that are dynamically evaluated using JSONLogic expressions and exposed through the GraphQL API. This feature enables packages to add domain-specific business logic and derived fields without modifying the core Reframe codebase.

### Key Features

- **Package-Specific**: Each package can define its own custom fields
- **JSONLogic-Based**: Fields are computed using JSONLogic expressions
- **Flexible Storage**: Three storage strategies (precompute, runtime, hybrid)
- **Typed GraphQL**: Package-specific GraphQL types for type safety
- **Filterable**: Full support for filtering and sorting by custom fields
- **Recomputable**: Ability to recompute fields with updated logic

## Use Cases

1. **Risk Scoring**: Compute transaction risk scores based on amount, counterparty, and other factors
2. **Categorization**: Classify transactions by type, region, or business rules
3. **Validation Flags**: Derive compliance or validation status fields
4. **Formatting**: Generate formatted summaries or display strings
5. **Time-Based**: Calculate age, expiration, or time-sensitive values
6. **Cross-Field Logic**: Combine multiple fields into derived insights

## Package Configuration

### api_config.json Structure

Each package can include an `api_config.json` file defining custom fields:

```json
{
  "custom_fields": [
    {
      "name": "transaction_risk_score",
      "description": "Computed risk score based on amount and counterparty",
      "type": "number",
      "storage": "precompute",
      "logic": {
        "if": [
          {">": [{"var": "context.data.amount"}, 100000]},
          100,
          {"if": [
            {">": [{"var": "context.data.amount"}, 50000]},
            70,
            30
          ]}
        ]
      }
    },
    {
      "name": "is_cross_border",
      "description": "Whether transaction crosses country borders",
      "type": "boolean",
      "storage": "precompute",
      "logic": {
        "!=": [
          {"var": "context.data.debtor_country"},
          {"var": "context.data.creditor_country"}
        ]
      }
    },
    {
      "name": "days_since_transform",
      "description": "Days since transformation occurred",
      "type": "number",
      "storage": "runtime",
      "logic": {
        "date_diff_days": [
          {"now": []},
          {"var": "context.metadata.timestamp"}
        ]
      }
    },
    {
      "name": "formatted_summary",
      "description": "Human-readable transaction summary",
      "type": "string",
      "storage": "hybrid",
      "logic": {
        "cat": [
          {"var": "context.data.debtor_name"},
          " → ",
          {"var": "context.data.creditor_name"},
          ": $",
          {"var": "context.data.amount"}
        ]
      }
    }
  ]
}
```

### Field Definition Schema

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `name` | string | Yes | Unique field name (use snake_case) |
| `description` | string | Yes | Human-readable description |
| `type` | string | Yes | Data type: `string`, `number`, `boolean`, or `json` |
| `storage` | string | Yes | Storage strategy: `precompute`, `runtime`, or `hybrid` |
| `logic` | object | Yes | JSONLogic expression to compute the field value |

## Storage Strategies

### Precompute

**When to use**: Fields used in filters/sorting, expensive computations, stable values

- **Computation**: At message storage time (before DB insert)
- **Storage**: Saved in `context.custom_fields.<field_name>`
- **Query**: Read directly from database (fast)
- **Recompute**: Can override with `recompute_custom_fields: true` flag

**Example**:
```json
{
  "name": "is_high_value",
  "type": "boolean",
  "storage": "precompute",
  "logic": {">": [{"var": "context.data.amount"}, 50000]}
}
```

### Runtime

**When to use**: Time-dependent fields, volatile computations, frequently changing logic

- **Computation**: Only at GraphQL query time
- **Storage**: Not stored in database
- **Query**: Evaluated fresh on every query
- **Recompute**: Always computed (flag has no effect)

**Example**:
```json
{
  "name": "days_since_transform",
  "type": "number",
  "storage": "runtime",
  "logic": {
    "date_diff_days": [
      {"now": []},
      {"var": "context.metadata.timestamp"}
    ]
  }
}
```

### Hybrid

**When to use**: Fields where logic might evolve, need both speed and flexibility

- **Computation**: Both storage time and query time
- **Storage**: Saved in `context.custom_fields.<field_name>` (cached)
- **Query**:
  - Default: Return stored value (fast)
  - With `recompute_custom_fields: true`: Re-evaluate with latest logic
- **Recompute**: Supports on-demand recomputation

**Example**:
```json
{
  "name": "compliance_status",
  "type": "string",
  "storage": "hybrid",
  "logic": {
    "if": [
      {"and": [
        {"var": "context.data.kyc_verified"},
        {"<": [{"var": "context.data.amount"}, 10000]}
      ]},
      "approved",
      "review_required"
    ]
  }
}
```

## JSONLogic Expressions

### Variable Access

Access message data using JSONPath-like syntax:

```json
{"var": "context.data.field_name"}           // Access data fields
{"var": "context.metadata.package_id"}       // Access metadata
{"var": "payload"}                           // Access original payload
{"var": "context.custom_fields.other_field"} // Access other custom fields
```

### Common Operators

#### Comparison
```json
{"==": [{"var": "context.data.currency"}, "USD"]}
{"!=": [{"var": "context.data.status"}, "cancelled"]}
{">": [{"var": "context.data.amount"}, 1000]}
{">=": [{"var": "context.data.amount"}, 1000]}
{"<": [{"var": "context.data.amount"}, 10000]}
{"<=": [{"var": "context.data.amount"}, 10000]}
```

#### Logical
```json
{"and": [
  {">": [{"var": "context.data.amount"}, 5000]},
  {"==": [{"var": "context.data.currency"}, "EUR"]}
]}

{"or": [
  {"==": [{"var": "context.data.priority"}, "high"]},
  {">": [{"var": "context.data.amount"}, 100000]}
]}

{"!": {"var": "context.data.is_internal"}}
```

#### Arithmetic
```json
{"+": [{"var": "context.data.amount"}, {"var": "context.data.fee"}]}
{"-": [{"var": "context.data.total"}, {"var": "context.data.tax"}]}
{"*": [{"var": "context.data.quantity"}, {"var": "context.data.price"}]}
{"/": [{"var": "context.data.total"}, {"var": "context.data.count"}]}
{"%": [{"var": "context.data.amount"}, 100]}
```

#### String Operations
```json
{"cat": ["Amount: $", {"var": "context.data.amount"}]}

{"in": ["USD", {"var": "context.data.currency"}]}

{"substr": [{"var": "context.data.reference"}, 0, 10]}
```

#### Conditionals
```json
{"if": [
  {">": [{"var": "context.data.amount"}, 50000]},
  "high",
  "low"
]}

{"if": [
  {">": [{"var": "context.data.amount"}, 100000]},
  "critical",
  {">": [{"var": "context.data.amount"}, 50000]},
  "high",
  "normal"
]}
```

#### Array Operations
```json
{"in": ["EUR", ["USD", "EUR", "GBP"]]}

{"map": [
  {"var": "context.data.items"},
  {"var": "price"}
]}

{"filter": [
  {"var": "context.data.items"},
  {">": [{"var": "quantity"}, 0]}
]}

{"reduce": [
  {"var": "context.data.items"},
  {"+": [{"var": "accumulator"}, {"var": "current.amount"}]},
  0
]}
```

### Custom Functions

#### Date/Time
```json
{"now": []}                                  // Current timestamp
{"date_diff_days": [date1, date2]}          // Days between dates
{"date_format": [{"var": "timestamp"}]}      // Format date string
```

#### Null Handling
```json
{"missing": ["context.data.field"]}         // Check if field is missing
{"missing_some": [1, ["field1", "field2"]]} // At least N fields missing
```

## GraphQL Integration

### Schema

Custom fields are exposed through a typed GraphQL interface:

```graphql
type TransformationMessage {
  id: String!
  payload: JSON!
  context: JSON!
  package_id: String

  # Custom fields based on package configuration
  customFields: CustomFieldsUnion
}

# Union type for different packages
union CustomFieldsUnion = SwiftCbprCustomFields

# Package-specific custom fields (auto-generated from api_config.json)
type SwiftCbprCustomFields {
  transaction_risk_score: Float
  is_cross_border: Boolean
  days_since_transform: Float
  formatted_summary: String
}
```

### Querying Custom Fields

#### Basic Query
```graphql
query {
  messages(limit: 10) {
    messages {
      id
      package_id
      customFields {
        ... on SwiftCbprCustomFields {
          transaction_risk_score
          is_cross_border
          formatted_summary
        }
      }
    }
  }
}
```

#### With Filtering
```graphql
query {
  messages(
    filter: {
      package_id: "swift-cbpr-mt-mx",
      custom_field_filters: {
        transaction_risk_score: { gte: 70 },
        is_cross_border: true
      }
    },
    limit: 50
  ) {
    messages {
      id
      customFields {
        ... on SwiftCbprCustomFields {
          transaction_risk_score
          is_cross_border
        }
      }
    }
    total_count
  }
}
```

#### With Recomputation
```graphql
query {
  messages(
    filter: { package_id: "swift-cbpr-mt-mx" },
    recompute_custom_fields: true,
    limit: 20
  ) {
    messages {
      id
      customFields {
        ... on SwiftCbprCustomFields {
          compliance_status  # Recomputed with latest logic
          formatted_summary  # Recomputed with latest logic
        }
      }
    }
  }
}
```

### Filter Operators

Custom field filters support MongoDB-style operators:

```graphql
custom_field_filters: {
  field_name: { eq: "value" }       # Equal
  field_name: { ne: "value" }       # Not equal
  field_name: { gt: 100 }           # Greater than
  field_name: { gte: 100 }          # Greater than or equal
  field_name: { lt: 1000 }          # Less than
  field_name: { lte: 1000 }         # Less than or equal
  field_name: { in: [1, 2, 3] }     # In array
  field_name: { exists: true }      # Field exists
}
```

## Database Storage

### Document Structure

Custom fields are stored in the `context.custom_fields` object:

```json
{
  "id": "msg-uuid-123",
  "payload": "...",
  "context": {
    "metadata": {
      "package_id": "swift-cbpr-mt-mx",
      "timestamp": "2025-01-15T10:30:00Z"
    },
    "data": {
      "amount": 75000,
      "debtor_name": "ACME Corp",
      "creditor_name": "XYZ Ltd"
    },
    "custom_fields": {
      "transaction_risk_score": 70,
      "is_cross_border": true,
      "formatted_summary": "ACME Corp → XYZ Ltd: $75000"
    }
  }
}
```

**Note**: Runtime-only fields are NOT stored in the database.

### Indexing

For optimal query performance, create indexes on frequently filtered custom fields:

```javascript
// Generic package_id index
db.reframe_audit.createIndex({"context.metadata.package_id": 1})

// Wildcard index for all custom fields (MongoDB 4.2+)
db.reframe_audit.createIndex({"context.custom_fields.$**": 1})

// Specific custom field indexes for better performance
db.reframe_audit.createIndex({"context.custom_fields.transaction_risk_score": 1})
db.reframe_audit.createIndex({"context.custom_fields.is_cross_border": 1})

// Compound indexes for common filter combinations
db.reframe_audit.createIndex({
  "context.metadata.package_id": 1,
  "context.custom_fields.transaction_risk_score": 1
})
```

## Implementation Architecture

### Message Flow

1. **Transformation Request** → Handler receives message
2. **Package Resolution** → Determine which package to use
3. **Metadata Injection** → Add `package_id` to message metadata
4. **Workflow Processing** → Execute transformation workflows
5. **Custom Field Computation** → Evaluate precompute/hybrid fields
6. **Storage** → Save message with computed custom fields to database
7. **GraphQL Query** → Resolve custom fields (use stored or recompute)

### Modules

#### `src/custom_fields.rs`
Core custom fields logic:
- JSONLogic evaluation engine
- Field computation at storage time
- Field computation at query time
- Error handling (null on failure)

#### `src/package_manager.rs`
Package configuration:
- Load `api_config.json` from packages
- Cache custom field definitions
- Validate field configurations

#### `src/graphql/schema.rs`
GraphQL integration:
- Custom field resolvers
- Package-specific type registration
- Filter and recompute support

#### `src/database/mongodb.rs`
Database operations:
- Custom field filter queries
- Index recommendations

## Error Handling

### JSONLogic Evaluation Errors

When a JSONLogic expression fails to evaluate:
- **Behavior**: Field returns `null`
- **Logging**: Error logged with field name and message ID
- **GraphQL**: Field appears as `null` in query results
- **No Failure**: Other fields continue to compute normally

Example:
```json
{
  "transaction_risk_score": null,  // Failed to compute
  "is_cross_border": true,          // Successfully computed
  "formatted_summary": "..."        // Successfully computed
}
```

### Missing Variable References

If a JSONLogic expression references a non-existent field:
- **Behavior**: Variable returns `null`, expression continues
- **Best Practice**: Use `missing` operator to check for fields

```json
{
  "if": [
    {"missing": ["context.data.amount"]},
    null,
    {">": [{"var": "context.data.amount"}, 50000]}
  ]
}
```

### Invalid Configuration

Package with invalid `api_config.json`:
- **Validation**: Checked during package load
- **Behavior**: Warning logged, package loads without custom fields
- **Queries**: Package has no custom fields available

## Best Practices

### Naming Conventions

- Use `snake_case` for field names
- Prefix domain-specific fields (e.g., `swift_network_code`)
- Keep names descriptive but concise

### Storage Strategy Selection

Choose storage type based on:

| Criteria | Precompute | Runtime | Hybrid |
|----------|------------|---------|--------|
| Used in filters/sorting | ✅ | ❌ | ✅ |
| Time-dependent | ❌ | ✅ | ❌ |
| Expensive computation | ✅ | ❌ | ✅ |
| Logic frequently changes | ❌ | ✅ | ✅ |
| Stable value | ✅ | ❌ | ⚠️ |

### Performance Optimization

1. **Use Precompute** for fields used in filters to avoid query-time overhead
2. **Index frequently filtered fields** in MongoDB
3. **Limit Runtime fields** as they compute on every query
4. **Batch recomputation** when updating logic (use background jobs)

### Security Considerations

1. **Avoid sensitive data** in custom fields (use access controls instead)
2. **Validate JSONLogic** expressions during package load
3. **Limit expression complexity** to prevent DoS attacks
4. **Sanitize field names** to prevent injection attacks

## Migration Guide

### Adding Custom Fields to Existing Package

1. Create `api_config.json` in package root:
```json
{
  "custom_fields": [
    {
      "name": "my_field",
      "description": "Description",
      "type": "string",
      "storage": "precompute",
      "logic": {"var": "context.data.field"}
    }
  ]
}
```

2. Reload package in Reframe:
```bash
curl -X POST http://localhost:3000/admin/reload-workflows
```

3. Verify field appears in GraphQL schema:
```graphql
query {
  __type(name: "YourPackageCustomFields") {
    fields {
      name
      type { name }
    }
  }
}
```

### Updating Field Logic

For **hybrid** storage fields:
1. Update JSONLogic in `api_config.json`
2. Reload package
3. Query with `recompute_custom_fields: true` to see new values
4. Optionally bulk-update database with background job

For **precompute** storage fields:
1. Update JSONLogic in `api_config.json`
2. Reload package
3. New messages use updated logic
4. Old messages retain original computed values (unless recomputed)

### Bulk Recomputation

To update historical data with new logic:

```javascript
// MongoDB aggregation pipeline to recompute (pseudo-code)
// This would be part of a background job/script
db.reframe_audit.find({
  "context.metadata.package_id": "swift-cbpr-mt-mx"
}).forEach(doc => {
  // Re-evaluate custom fields with updated logic
  // Update document
});
```

## Examples

### Risk Scoring
```json
{
  "name": "risk_level",
  "type": "string",
  "storage": "precompute",
  "logic": {
    "if": [
      {">": [{"var": "context.data.amount"}, 100000]},
      "critical",
      {">": [{"var": "context.data.amount"}, 50000]},
      "high",
      {">": [{"var": "context.data.amount"}, 10000]},
      "medium",
      "low"
    ]
  }
}
```

### Regional Categorization
```json
{
  "name": "region",
  "type": "string",
  "storage": "precompute",
  "logic": {
    "if": [
      {"in": [{"var": "context.data.country"}, ["US", "CA", "MX"]]},
      "americas",
      {"in": [{"var": "context.data.country"}, ["GB", "DE", "FR", "IT"]]},
      "europe",
      {"in": [{"var": "context.data.country"}, ["JP", "CN", "SG"]]},
      "apac",
      "other"
    ]
  }
}
```

### Compliance Check
```json
{
  "name": "requires_approval",
  "type": "boolean",
  "storage": "hybrid",
  "logic": {
    "or": [
      {">": [{"var": "context.data.amount"}, 50000]},
      {"==": [{"var": "context.data.is_sanctioned_country"}, true]},
      {"==": [{"var": "context.data.kyc_status"}, "pending"]}
    ]
  }
}
```

### Formatted Display
```json
{
  "name": "display_name",
  "type": "string",
  "storage": "hybrid",
  "logic": {
    "cat": [
      "[",
      {"var": "context.data.message_type"},
      "] ",
      {"var": "context.data.debtor_name"},
      " → ",
      {"var": "context.data.creditor_name"},
      " (",
      {"var": "context.data.currency"},
      " ",
      {"var": "context.data.amount"},
      ")"
    ]
  }
}
```

## Troubleshooting

### Custom Fields Not Appearing

**Symptoms**: GraphQL query returns `null` for `customFields`

**Possible Causes**:
1. Package doesn't have `api_config.json`
2. Package not reloaded after adding config
3. Message missing `package_id` in metadata

**Solutions**:
1. Verify `api_config.json` exists in package root
2. Call `/admin/reload-workflows` endpoint
3. Check message `context.metadata.package_id` field

### Field Always Returns Null

**Symptoms**: Specific custom field always `null`

**Possible Causes**:
1. JSONLogic expression error
2. Referenced variable doesn't exist
3. Type mismatch in expression

**Solutions**:
1. Check application logs for JSONLogic errors
2. Verify variable paths exist in message context
3. Test expression with sample data

### Poor Query Performance

**Symptoms**: GraphQL queries with custom field filters are slow

**Possible Causes**:
1. Missing database indexes
2. Too many runtime-computed fields
3. Complex JSONLogic expressions

**Solutions**:
1. Add indexes on filtered custom fields
2. Change storage type to `precompute` or `hybrid`
3. Simplify or optimize JSONLogic expressions

### Recomputation Not Working

**Symptoms**: `recompute_custom_fields: true` returns same values

**Possible Causes**:
1. Field storage type is `precompute` (can't recompute)
2. Package not reloaded with updated logic
3. Querying wrong package

**Solutions**:
1. Change storage type to `hybrid` for recomputable fields
2. Reload package after logic changes
3. Verify `package_id` filter matches updated package
