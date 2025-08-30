# Vertical Scaling Architecture for Reframe

## Executive Summary

This document outlines a minimal yet effective approach to enable Reframe to utilize maximum available resources through vertical scaling. The current architecture has a critical bottleneck where only one request can be processed at a time due to mutex-based engine locking.

## Current Architecture Issues

### 1. Single-Threaded Engine Access
- **Problem**: Engines wrapped in `Arc<Mutex<Engine>>` create serialization
- **Impact**: Only ONE request processes at a time, regardless of available CPU cores
- **Result**: ~50-100 requests/second on multi-core machines

### 2. Default Tokio Configuration
- **Problem**: No explicit worker thread configuration
- **Impact**: Not utilizing all available CPU cores
- **Result**: Suboptimal async task scheduling

### 3. No Request Parallelism
- **Problem**: Each transformation locks the entire engine
- **Impact**: Requests queue sequentially
- **Result**: High latency under load

## Proposed Architecture

### Overview
```
┌─────────────────────────────────────────────────┐
│                  HTTP Request                   │
└────────────────────┬────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────┐
│          Axum Router + Middleware               │
│  - Concurrency Limit (4x CPU cores)             │
│  - Request Buffering (1000 requests)            │
└────────────────────┬────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────┐
│         Tokio Runtime (Multi-threaded)          │
│  - Worker threads = CPU cores                   │
│  - Optimized stack size                         │
└────────────────────┬────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────┐
│              Engine Pool                        │
│  - Size = 2x CPU cores                          │
│  - Lock-free queue                              │
│  - Fair scheduling                              │
└────────────────────┬────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────┐
│         Concurrent Processing                   │
│  - Multiple engines process simultaneously      │
│  - No blocking on engine access                 │
└─────────────────────────────────────────────────┘
```

## Implementation Plan

### Phase 1: Engine Pool Architecture

#### 1.1 Engine Pool Structure
```rust
// src/engine_pool.rs
pub struct EnginePool {
    engines: Arc<ArrayQueue<Arc<Engine>>>,
    semaphore: Arc<Semaphore>,
    metrics: Arc<PoolMetrics>,
}

pub struct PoolMetrics {
    pub total_engines: usize,
    pub available_engines: AtomicUsize,
    pub total_requests: AtomicU64,
    pub active_requests: AtomicUsize,
}
```

#### 1.2 Pool Configuration
- **Default size**: 2 × CPU cores
- **Minimum**: 4 engines
- **Maximum**: 64 engines
- **Queue type**: Lock-free `ArrayQueue` from crossbeam

#### 1.3 Benefits
- Eliminates mutex contention
- Enables true parallel processing
- Fair distribution of requests

### Phase 2: Tokio Runtime Configuration

#### 2.1 Multi-threaded Runtime
```rust
// src/main.rs
fn main() {
    let config = load_config();
    let num_workers = config.worker_threads
        .unwrap_or_else(|| num_cpus::get());
    
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num_workers)
        .thread_name("reframe-worker")
        .thread_stack_size(2 * 1024 * 1024) // 2MB stack
        .max_blocking_threads(512)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            run_server(config).await
        });
}
```

#### 2.2 Configuration Options
- **Worker threads**: Defaults to CPU core count
- **Thread stack**: 2MB (reduced from default 8MB)
- **Blocking threads**: 512 for I/O operations
- **Thread naming**: For easier debugging

### Phase 3: Concurrent Request Processing

#### 3.1 Request Handler Refactoring
```rust
// src/handlers.rs
pub async fn transform_mt_to_mx(
    State(state): State<AppState>,
    Json(request): Json<TransformationRequest>,
) -> Result<Json<TransformationResponse>, StatusCode> {
    // Get engine from pool (non-blocking)
    let engine = state.forward_engine_pool
        .acquire()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    
    // Process message
    let result = tokio::spawn(async move {
        engine.process_message(message).await
    }).await?;
    
    // Engine automatically returned to pool on drop
    Ok(Json(response))
}
```

#### 3.2 Pool Acquisition Strategy
- **Timeout**: 30 seconds default
- **Retry**: Exponential backoff
- **Fallback**: Return 503 if pool exhausted

### Phase 4: Connection Pooling & Backpressure

#### 4.1 Tower Middleware Stack
```rust
// src/main.rs
use tower::ServiceBuilder;
use tower::limit::ConcurrencyLimitLayer;
use tower::buffer::BufferLayer;
use tower::timeout::TimeoutLayer;

let app = Router::new()
    .route("/transform/mt-to-mx", post(transform_mt_to_mx))
    .layer(
        ServiceBuilder::new()
            // Timeout per request
            .layer(TimeoutLayer::new(Duration::from_secs(60)))
            // Buffer requests
            .layer(BufferLayer::new(1000))
            // Limit concurrent requests
            .layer(ConcurrencyLimitLayer::new(num_cores * 4))
            // Add request ID
            .layer(middleware::from_fn(correlation_middleware))
    );
```

#### 4.2 Backpressure Configuration
- **Buffer size**: 1000 requests
- **Concurrency limit**: 4 × CPU cores
- **Request timeout**: 60 seconds
- **Connection timeout**: 30 seconds

### Phase 5: Configuration Management

#### 5.1 Configuration Structure
```toml
# config.toml
[server]
port = 3000
host = "0.0.0.0"

[runtime]
worker_threads = 16              # Override auto-detection
max_blocking_threads = 512

[engine_pool]
forward_pool_size = 32           # Forward transformation engines
reverse_pool_size = 32           # Reverse transformation engines
acquisition_timeout_ms = 30000

[limits]
max_concurrent_requests = 64
request_buffer_size = 1000
request_timeout_secs = 60
max_request_body_size = 10485760  # 10MB

[monitoring]
enable_metrics = true
metrics_port = 9090
```

#### 5.2 Environment Variable Overrides
```bash
REFRAME_WORKERS=16
REFRAME_FORWARD_POOL_SIZE=32
REFRAME_REVERSE_POOL_SIZE=32
REFRAME_MAX_CONCURRENT=64
REFRAME_BUFFER_SIZE=2000
```

## Performance Expectations

### Before Optimization
- **Throughput**: ~50-100 req/s
- **CPU Usage**: 6-12% (single core)
- **Latency P99**: 500ms-2s under load
- **Concurrency**: 1 request at a time

### After Optimization
- **Throughput**: ~2,000-5,000 req/s
- **CPU Usage**: 70-90% (all cores)
- **Latency P99**: 50-200ms under load
- **Concurrency**: 64+ simultaneous requests

### Scaling Formula
```
Max Throughput = (Pool Size × Avg Processing Time) / Request Time
Example: (32 engines × 1000ms) / 20ms = 1,600 req/s per engine type
```

## Monitoring & Metrics

### Key Metrics to Track
1. **Engine Pool Metrics**
   - Available engines
   - Acquisition wait time
   - Pool exhaustion events

2. **Request Metrics**
   - Active requests
   - Request queue depth
   - Processing time histogram

3. **System Metrics**
   - CPU utilization per core
   - Memory usage
   - Thread count

### Health Check Endpoint
```json
GET /health
{
  "status": "healthy",
  "version": "3.0.6",
  "engines": {
    "forward": {
      "total": 32,
      "available": 28,
      "active_requests": 4
    },
    "reverse": {
      "total": 32,
      "available": 30,
      "active_requests": 2
    }
  },
  "system": {
    "cpu_cores": 16,
    "worker_threads": 16,
    "memory_used_mb": 512,
    "uptime_seconds": 3600
  }
}
```

## Testing Strategy

### Load Testing
```bash
# Install Apache Bench
apt-get install apache2-utils

# Test concurrent load
ab -n 10000 -c 100 -p sample.json -T application/json \
   http://localhost:3000/transform/mt-to-mx

# Monitor during test
watch -n 1 'curl -s localhost:3000/health | jq .'
```

### Stress Testing
```bash
# Gradual load increase
for c in 10 20 40 80 160; do
  echo "Testing with $c concurrent requests"
  ab -n 1000 -c $c -p sample.json -T application/json \
     http://localhost:3000/transform/mt-to-mx
  sleep 5
done
```

## Rollout Plan

### Step 1: Prepare Dependencies
- Add required crates to Cargo.toml
- Create configuration module
- Set up metrics collection

### Step 2: Implement Engine Pool
- Create engine_pool.rs module
- Refactor AppState to use pools
- Update engine initialization

### Step 3: Update Runtime
- Configure Tokio runtime
- Add worker thread configuration
- Implement graceful shutdown

### Step 4: Refactor Handlers
- Update all transformation handlers
- Add pool acquisition logic
- Implement timeout handling

### Step 5: Add Middleware
- Configure Tower middleware stack
- Add backpressure controls
- Implement request buffering

### Step 6: Testing & Validation
- Unit tests for engine pool
- Integration tests for concurrent requests
- Load testing to validate improvements

## Risk Mitigation

### Potential Issues & Solutions

1. **Memory Growth**
   - Monitor heap usage
   - Implement engine recycling after N uses
   - Add memory limits per engine

2. **Engine State Corruption**
   - Ensure engines are stateless
   - Reset engine state between uses
   - Add validation after each use

3. **Pool Exhaustion**
   - Implement circuit breaker
   - Add adaptive pool sizing
   - Monitor and alert on exhaustion

4. **Uneven Load Distribution**
   - Use fair scheduling in pool
   - Monitor engine usage patterns
   - Implement work stealing if needed

## Success Criteria

- [ ] 10x throughput improvement
- [ ] >70% CPU utilization under load
- [ ] P99 latency <200ms at 80% capacity
- [ ] Zero request drops under normal load
- [ ] Graceful degradation under overload

## Conclusion

This minimal approach provides significant performance improvements while maintaining code simplicity and reliability. The architecture scales automatically with available hardware resources and can be tuned via configuration without code changes.