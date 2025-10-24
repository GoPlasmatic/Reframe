// MongoDB Indexes for Reframe Aggregation API
// Run this script using: mongosh reframe < mongodb_indexes.js

// Use the correct database
db = db.getSiblingDB('reframe');

print('Creating indexes for reframe_audit collection...');

// ============================================================================
// BASE INDEXES - Essential for all queries
// ============================================================================

print('\n1. Creating base indexes...');

// Timestamp index (DESC for recent-first sorting)
db.reframe_audit.createIndex(
  { "timestamp": -1 },
  { name: "idx_timestamp", background: true }
);
print('✓ Created timestamp index');

// Message ID index (for direct lookups)
db.reframe_audit.createIndex(
  { "id": 1 },
  { name: "idx_id", unique: true, background: true }
);
print('✓ Created id index');

// Package ID index (filter by package)
db.reframe_audit.createIndex(
  { "context.metadata.package_id": 1 },
  { name: "idx_package_id", background: true }
);
print('✓ Created package_id index');

// Message type hint index
db.reframe_audit.createIndex(
  { "context.metadata.message_type_hint": 1 },
  { name: "idx_message_type_hint", background: true }
);
print('✓ Created message_type_hint index');

// Direction index
db.reframe_audit.createIndex(
  { "context.metadata.direction": 1 },
  { name: "idx_direction", background: true }
);
print('✓ Created direction index');

// Errors index (for success/failure filtering)
db.reframe_audit.createIndex(
  { "errors": 1 },
  { name: "idx_errors", sparse: true, background: true }
);
print('✓ Created errors index');

// ============================================================================
// COMPOUND INDEXES - For common aggregation patterns
// ============================================================================

print('\n2. Creating compound indexes for aggregations...');

// Package + Timestamp (most common aggregation filter)
db.reframe_audit.createIndex(
  {
    "context.metadata.package_id": 1,
    "timestamp": -1
  },
  { name: "idx_package_timestamp", background: true }
);
print('✓ Created package_id + timestamp compound index');

// Message Type + Timestamp (for message type trends)
db.reframe_audit.createIndex(
  {
    "context.metadata.message_type_hint": 1,
    "timestamp": -1
  },
  { name: "idx_msgtype_timestamp", background: true }
);
print('✓ Created message_type + timestamp compound index');

// Direction + Timestamp (for directional analysis)
db.reframe_audit.createIndex(
  {
    "context.metadata.direction": 1,
    "timestamp": -1
  },
  { name: "idx_direction_timestamp", background: true }
);
print('✓ Created direction + timestamp compound index');

// Package + Message Type + Timestamp (for detailed filtering)
db.reframe_audit.createIndex(
  {
    "context.metadata.package_id": 1,
    "context.metadata.message_type_hint": 1,
    "timestamp": -1
  },
  { name: "idx_package_msgtype_timestamp", background: true }
);
print('✓ Created package_id + message_type + timestamp compound index');

// Errors + Timestamp (for error trend analysis)
db.reframe_audit.createIndex(
  {
    "errors": 1,
    "timestamp": -1
  },
  { name: "idx_errors_timestamp", sparse: true, background: true }
);
print('✓ Created errors + timestamp compound index');

// ============================================================================
// WILDCARD INDEX - For dynamic custom fields (MongoDB 4.2+)
// ============================================================================

print('\n3. Creating wildcard index for custom fields...');

// Package-specific wildcard index (swift-cbpr-mt-mx)
db.reframe_audit.createIndex(
  { "context.swiftCbprMtMxFields.$**": 1 },
  { name: "idx_swift_cbpr_fields_wildcard", background: true }
);
print('✓ Created wildcard index for swift-cbpr-mt-mx custom fields');

// ============================================================================
// CUSTOM FIELD INDEXES - For SWIFT CBPR package (common aggregations)
// ============================================================================

print('\n4. Creating specific custom field indexes (SWIFT CBPR)...');

// Amount index (for financial aggregations)
db.reframe_audit.createIndex(
  { "context.swiftCbprMtMxFields.amount": 1 },
  { name: "idx_cf_amount", sparse: true, background: true }
);
print('✓ Created swiftCbprMtMxFields.amount index');

// Currency index (for currency grouping)
db.reframe_audit.createIndex(
  { "context.swiftCbprMtMxFields.currency": 1 },
  { name: "idx_cf_currency", sparse: true, background: true }
);
print('✓ Created swiftCbprMtMxFields.currency index');

// Message Type index (for message type breakdown)
db.reframe_audit.createIndex(
  { "context.swiftCbprMtMxFields.messageType": 1 },
  { name: "idx_cf_messageType", sparse: true, background: true }
);
print('✓ Created swiftCbprMtMxFields.messageType index');

// Sender BIC index (for sender analysis)
db.reframe_audit.createIndex(
  { "context.swiftCbprMtMxFields.senderBic": 1 },
  { name: "idx_cf_senderBic", sparse: true, background: true }
);
print('✓ Created swiftCbprMtMxFields.senderBic index');

// Receiver BIC index (for receiver analysis)
db.reframe_audit.createIndex(
  { "context.swiftCbprMtMxFields.receiverBic": 1 },
  { name: "idx_cf_receiverBic", sparse: true, background: true }
);
print('✓ Created swiftCbprMtMxFields.receiverBic index');

// Sender Country index (for geographic analysis)
db.reframe_audit.createIndex(
  { "context.swiftCbprMtMxFields.senderCountry": 1 },
  { name: "idx_cf_senderCountry", sparse: true, background: true }
);
print('✓ Created swiftCbprMtMxFields.senderCountry index');

// Receiver Country index (for geographic analysis)
db.reframe_audit.createIndex(
  { "context.swiftCbprMtMxFields.receiverCountry": 1 },
  { name: "idx_cf_receiverCountry", sparse: true, background: true }
);
print('✓ Created swiftCbprMtMxFields.receiverCountry index');

// Risk Category index (for risk analysis)
db.reframe_audit.createIndex(
  { "context.swiftCbprMtMxFields.riskCategory": 1 },
  { name: "idx_cf_riskCategory", sparse: true, background: true }
);
print('✓ Created swiftCbprMtMxFields.riskCategory index');

// Amount Value Band index (for amount distribution)
db.reframe_audit.createIndex(
  { "context.swiftCbprMtMxFields.amountValueBand": 1 },
  { name: "idx_cf_amountValueBand", sparse: true, background: true }
);
print('✓ Created swiftCbprMtMxFields.amountValueBand index');

// Processing Priority index (for priority analysis)
db.reframe_audit.createIndex(
  { "context.swiftCbprMtMxFields.processingPriority": 1 },
  { name: "idx_cf_processingPriority", sparse: true, background: true }
);
print('✓ Created swiftCbprMtMxFields.processingPriority index');

// Direction index (custom field version)
db.reframe_audit.createIndex(
  { "context.swiftCbprMtMxFields.direction": 1 },
  { name: "idx_cf_direction", sparse: true, background: true }
);
print('✓ Created swiftCbprMtMxFields.direction index');

// ============================================================================
// COMPOUND CUSTOM FIELD INDEXES - For common multi-field aggregations
// ============================================================================

print('\n5. Creating compound custom field indexes...');

// Sender BIC + Amount (for sender volume analysis)
db.reframe_audit.createIndex(
  {
    "context.swiftCbprMtMxFields.senderBic": 1,
    "context.swiftCbprMtMxFields.amount": 1
  },
  { name: "idx_cf_senderBic_amount", sparse: true, background: true }
);
print('✓ Created senderBic + amount compound index');

// Message Type + Risk Category (for risk breakdown by type)
db.reframe_audit.createIndex(
  {
    "context.swiftCbprMtMxFields.messageType": 1,
    "context.swiftCbprMtMxFields.riskCategory": 1
  },
  { name: "idx_cf_msgType_risk", sparse: true, background: true }
);
print('✓ Created messageType + riskCategory compound index');

// Sender Country + Receiver Country (for cross-border analysis)
db.reframe_audit.createIndex(
  {
    "context.swiftCbprMtMxFields.senderCountry": 1,
    "context.swiftCbprMtMxFields.receiverCountry": 1
  },
  { name: "idx_cf_senderCountry_receiverCountry", sparse: true, background: true }
);
print('✓ Created senderCountry + receiverCountry compound index');

// Currency + Amount (for currency volume analysis)
db.reframe_audit.createIndex(
  {
    "context.swiftCbprMtMxFields.currency": 1,
    "context.swiftCbprMtMxFields.amount": 1
  },
  { name: "idx_cf_currency_amount", sparse: true, background: true }
);
print('✓ Created currency + amount compound index');

// ============================================================================
// TEXT INDEX - For full-text search (optional)
// ============================================================================

print('\n6. Creating text index for search...');

db.reframe_audit.createIndex(
  {
    "payload": "text",
    "context.data": "text"
  },
  {
    name: "idx_text_search",
    background: true,
    weights: {
      "payload": 10,
      "context.data": 5
    }
  }
);
print('✓ Created text search index');

// ============================================================================
// PERFORMANCE OPTIMIZATION INDEXES
// ============================================================================

print('\n7. Creating performance optimization indexes...');

// Processing time index (for latency analysis)
db.reframe_audit.createIndex(
  { "context.metadata.processing_time_ms": 1 },
  { name: "idx_processing_time", sparse: true, background: true }
);
print('✓ Created processing_time_ms index');

// Package + Custom Fields for filtered aggregations
db.reframe_audit.createIndex(
  {
    "context.metadata.package_id": 1,
    "context.swiftCbprMtMxFields.messageType": 1,
    "timestamp": -1
  },
  { name: "idx_package_cf_msgtype_timestamp", sparse: true, background: true }
);
print('✓ Created package + custom field messageType + timestamp index');

// ============================================================================
// VERIFY INDEXES
// ============================================================================

print('\n8. Verifying created indexes...');
const indexes = db.reframe_audit.getIndexes();
print('\nTotal indexes created: ' + indexes.length);
print('\nIndex list:');
indexes.forEach(function(idx) {
  print('  - ' + idx.name + ': ' + JSON.stringify(idx.key));
});

// ============================================================================
// INDEX USAGE RECOMMENDATIONS
// ============================================================================

print('\n' + '='.repeat(80));
print('INDEX USAGE RECOMMENDATIONS');
print('='.repeat(80));

print(`
1. MONITORING
   - Monitor index usage: db.reframe_audit.aggregate([{$indexStats: {}}])
   - Check slow queries: db.setProfilingLevel(1, { slowms: 100 })
   - Review query plans: db.reframe_audit.find(...).explain("executionStats")

2. MAINTENANCE
   - Rebuild indexes periodically: db.reframe_audit.reIndex()
   - Monitor index size: db.reframe_audit.stats().indexSizes
   - Drop unused indexes after monitoring query patterns

3. OPTIMIZATION TIPS
   - Indexes are built in background mode to avoid blocking
   - Sparse indexes are used for custom fields (not all messages have them)
   - Consider partitioning if collection exceeds 100M documents
   - Use covered queries when possible (all fields in index)

4. MEMORY CONSIDERATIONS
   - Keep working set in RAM: db.serverStatus().wiredTiger.cache
   - Adjust cache size if needed: storage.wiredTiger.engineConfig.cacheSizeGB
   - Monitor index sizes vs available RAM

5. AGGREGATION PERFORMANCE
   - Use $match early in pipeline to reduce document processing
   - Indexes are used for $match and $sort stages
   - $group and aggregation functions run in-memory after filtering
   - Consider $out stage for materialized views of common aggregations

6. CUSTOM FIELDS
   - Add more specific indexes based on your query patterns
   - Wildcard index ($**) covers all fields but less efficient
   - Specific indexes (e.g., amount, currency) are faster for those fields
   - Monitor which custom fields are queried most and index them
`);

print('\n✅ All indexes created successfully!');
print('   MongoDB version required: 4.2+ (for wildcard indexes)');
print('   MongoDB version required: 4.4+ (for $dateTrunc in aggregations)');
print('   MongoDB version required: 5.0+ (for $percentile in aggregations)');
print('\n📊 You can now run high-performance aggregation queries!\n');
