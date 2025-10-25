# GraphQL API Migration Guide

## Overview

The Reframe GraphQL API has been redesigned with a unified query approach, consolidating 5 separate queries into a single, powerful `messages` query. This migration guide explains the changes and how to update your existing queries.

## Key Changes

### ✅ What's New
- **Single Unified Query**: All functionality now available through the `messages` query
- **Union Type Response**: Returns either `MessageConnection` (list mode) or `AggregationResult` (aggregate mode)
- **PathFilter Support**: Dynamic field filtering using dot notation
- **Enhanced Sorting**: Sort by any field in ascending or descending order
- **Mode Detection**: Automatically switches between list and aggregate modes based on parameters

### ❌ What's Removed
The following queries have been deprecated and removed:
- `message(id)` - Single message lookup
- `searchMessages(query)` - Text search
- `stats` - Statistics query
- `aggregate` - Separate aggregation query

## Migration Examples

### 1. Single Message by ID

**Before:**
```graphql
query {
  message(id: "msg-123") {
    id
    payload
    errors { code message }
  }
}
```

**After:**
```graphql
query {
  messages(filter: { path: [{ field: "id", value: { string: { eq: "msg-123" } } }] }, limit: 1) {
    ... on MessageConnection {
      messages {
        id
        payload
        errors { code message }
      }
    }
  }
}
```

### 2. Search Messages

**Before:**
```graphql
query {
  searchMessages(query: "payment", limit: 10) {
    id
    context
  }
}
```

**After:**
```graphql
query {
  messages(filter: { search: "payment" }, limit: 10) {
    ... on MessageConnection {
      messages {
        id
        context
      }
    }
  }
}
```

### 3. Get Statistics

**Before:**
```graphql
query {
  stats {
    totalMessages
    successCount
    failureCount
    successRate
  }
}
```

**After:**
```graphql
query {
  messages(
    groupBy: [{ field: "success" }],
    metrics: [
      { field: "id", function: COUNT, as: "count" }
    ]
  ) {
    ... on AggregationResult {
      data {
        group
        metrics
      }
    }
  }
}
```

### 4. Aggregation Queries

**Before:**
```graphql
query {
  aggregate(
    filter: { hasErrors: true },
    groupBy: [{ field: "packageId" }],
    metrics: [{ field: "id", function: COUNT, as: "error_count" }]
  ) {
    data {
      group
      metrics
    }
  }
}
```

**After:**
```graphql
query {
  messages(
    filter: { path: [{ field: "errors", value: { exists: true } }] },
    groupBy: [{ field: "packageId" }],
    metrics: [{ field: "id", function: COUNT, as: "error_count" }]
  ) {
    ... on AggregationResult {
      data {
        group
        metrics
      }
    }
  }
}
```

## Unified Query Parameters

### List Mode Parameters
- `filter`: Filter criteria (MessageFilter)
- `sort`: Array of sort configurations
- `limit`: Maximum messages to return (default: 50, max: 1000)
- `offset`: Pagination offset (default: 0)
- `recomputeCustomFields`: Force recalculation (default: false)

### Aggregation Mode Parameters
- `filter`: Filter criteria
- `groupBy`: Array of grouping configurations
- `metrics`: Array of metric calculations (REQUIRED for aggregation mode)
- `sort`: Sort aggregated results
- `limit`: Maximum groups to return

## Filter Options

### MessageFilter
```graphql
input MessageFilter {
  messageType: String
  direction: String
  success: Boolean
  dateFrom: DateTime
  dateTo: DateTime
  search: String
  packageId: String
  customFieldFilters: JSON
  path: [PathFilter]  # Dynamic field filtering
}
```

### PathFilter (New)
```graphql
input PathFilter {
  field: String!  # Dot notation path (e.g., "context.metadata.direction")
  value: FilterValue!
}

input FilterValue {
  string: StringFilter
  number: NumberFilter
  boolean: Boolean
  date: DateFilter
  exists: Boolean
}
```

## Common Patterns

### List Messages with Filters and Sorting
```graphql
query {
  messages(
    filter: { success: true, direction: "outgoing" },
    sort: [{ field: "timestamp", direction: DESC }],
    limit: 20
  ) {
    ... on MessageConnection {
      messages {
        id
        packageId
        timestamp
      }
      totalCount
      hasNextPage
    }
  }
}
```

### Aggregation with Time Buckets
```graphql
query {
  messages(
    groupBy: [{
      field: "timestamp",
      timeBucket: { interval: "1h" }
    }],
    metrics: [
      { field: "id", function: COUNT, as: "messages_per_hour" }
    ]
  ) {
    ... on AggregationResult {
      data {
        group
        metrics
      }
      totalGroups
      executionTimeMs
    }
  }
}
```

### Complex Filtering with PathFilter
```graphql
query {
  messages(
    filter: {
      path: [
        { field: "context.metadata.direction", value: { string: { eq: "outgoing" } } },
        { field: "errors", value: { exists: false } }
      ]
    }
  ) {
    ... on MessageConnection {
      messages { id }
      totalCount
    }
  }
}
```

## Tips and Best Practices

1. **Always use inline fragments** with the Union type response:
   - `... on MessageConnection` for list results
   - `... on AggregationResult` for aggregation results

2. **Include `__typename`** to identify the response type programmatically:
   ```graphql
   messages(...) {
     __typename
     ... on MessageConnection { ... }
     ... on AggregationResult { ... }
   }
   ```

3. **Mode Detection**: The query automatically switches to aggregation mode when `metrics` parameter is provided.

4. **Performance**: Use specific filters and limits to optimize query performance.

5. **Pagination**: Use `offset` and `limit` for paginating through large result sets in list mode.

## Known Limitations

- **Aggregation with dot notation**: Currently, MongoDB aggregation doesn't support dot notation in field paths for groupBy. Use top-level fields or consider data structure adjustments.

## Support

For questions or issues with the migration, please refer to the documentation or open an issue on GitHub.