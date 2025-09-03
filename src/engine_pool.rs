use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tracing::{debug, info, warn};

use crate::engine::{initialize_forward_engine, initialize_reverse_engine};
use crate::types::EngineWrapper;

/// Configuration for the engine pool
#[derive(Clone, Debug)]
pub struct PoolConfig {
    /// Number of engines in the pool
    pub pool_size: usize,
    /// Maximum time to wait for an available engine (milliseconds)
    pub timeout_ms: u64,
    /// Whether this is for forward (MT to MX) or reverse (MX to MT) transformation
    pub is_forward: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            pool_size: num_cpus::get(),
            timeout_ms: 30000,
            is_forward: true,
        }
    }
}

/// Metrics for monitoring pool performance
#[derive(Debug, Default)]
pub struct PoolMetrics {
    pub total_acquisitions: std::sync::atomic::AtomicU64,
    pub active_engines: std::sync::atomic::AtomicU64,
    pub wait_time_total_ms: std::sync::atomic::AtomicU64,
    pub timeouts: std::sync::atomic::AtomicU64,
}

/// A pool of transformation engines for concurrent processing
pub struct EnginePool {
    /// Vector of engine instances, each protected by a Mutex for exclusive access
    engines: Vec<Arc<Mutex<Arc<EngineWrapper>>>>,
    /// Semaphore for controlling concurrent access
    semaphore: Arc<Semaphore>,
    /// Pool configuration
    config: PoolConfig,
    /// Performance metrics
    metrics: Arc<PoolMetrics>,
}

impl EnginePool {
    /// Creates a new engine pool with the specified configuration
    pub async fn new(config: PoolConfig) -> Result<Self, Box<dyn std::error::Error>> {
        info!(
            "Initializing {} engine pool with {} engines",
            if config.is_forward {
                "forward"
            } else {
                "reverse"
            },
            config.pool_size
        );

        let mut engines = Vec::with_capacity(config.pool_size);

        // Create multiple engine instances
        for i in 0..config.pool_size {
            debug!("Creating engine instance {}/{}", i + 1, config.pool_size);

            let engine = if config.is_forward {
                initialize_forward_engine().await?
            } else {
                initialize_reverse_engine().await?
            };

            engines.push(Arc::new(Mutex::new(engine)));
        }

        let semaphore = Arc::new(Semaphore::new(config.pool_size));

        info!(
            "Engine pool initialized successfully with {} engines",
            config.pool_size
        );

        Ok(Self {
            engines,
            semaphore,
            config,
            metrics: Arc::new(PoolMetrics::default()),
        })
    }

    /// Acquires an engine from the pool
    pub async fn acquire(&self) -> Result<PooledEngine, Box<dyn std::error::Error>> {
        let start = Instant::now();

        // Update acquisition metrics
        self.metrics
            .total_acquisitions
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // Try to acquire permit with timeout
        let permit = tokio::time::timeout(
            Duration::from_millis(self.config.timeout_ms),
            self.semaphore.clone().acquire_owned(),
        )
        .await
        .map_err(|_| {
            self.metrics
                .timeouts
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            format!(
                "Timeout waiting for available engine after {}ms",
                self.config.timeout_ms
            )
        })?
        .map_err(|e| format!("Failed to acquire semaphore permit: {}", e))?;

        let wait_time = start.elapsed();
        self.metrics.wait_time_total_ms.fetch_add(
            wait_time.as_millis() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );

        if wait_time.as_millis() > 100 {
            warn!("Engine acquisition took {}ms", wait_time.as_millis());
        }

        // Get engine index based on permit (round-robin)
        let index = self.get_next_engine_index();
        let engine_mutex = self.engines[index].clone();

        self.metrics
            .active_engines
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        debug!("Engine {} acquired after {:?}", index, wait_time);

        Ok(PooledEngine {
            engine_mutex,
            _permit: permit,
            pool: self.clone(),
            index,
        })
    }

    /// Gets the next engine index (simple round-robin for now)
    fn get_next_engine_index(&self) -> usize {
        // For now, use a simple approach - the semaphore ensures we don't exceed pool size
        // In production, could use atomic counter for true round-robin
        let active = self
            .metrics
            .active_engines
            .load(std::sync::atomic::Ordering::Relaxed) as usize;
        active % self.config.pool_size
    }

    /// Reloads all engines in the pool with new workflows
    pub async fn reload_all(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Reloading all engines in the pool");

        // Create new engines
        let mut new_engines = Vec::with_capacity(self.config.pool_size);

        for i in 0..self.config.pool_size {
            debug!(
                "Reloading engine instance {}/{}",
                i + 1,
                self.config.pool_size
            );

            let engine = if self.config.is_forward {
                initialize_forward_engine().await?
            } else {
                initialize_reverse_engine().await?
            };

            new_engines.push(Arc::new(Mutex::new(engine)));
        }

        // Atomically swap the engines
        self.engines = new_engines;

        info!("Successfully reloaded {} engines", self.config.pool_size);
        Ok(())
    }

    /// Gets current pool metrics
    pub fn metrics(&self) -> PoolStats {
        let active = self
            .metrics
            .active_engines
            .load(std::sync::atomic::Ordering::Relaxed);

        PoolStats {
            pool_size: self.config.pool_size,
            active_engines: active as usize,
        }
    }
}

impl Clone for EnginePool {
    fn clone(&self) -> Self {
        Self {
            engines: self.engines.clone(),
            semaphore: self.semaphore.clone(),
            config: self.config.clone(),
            metrics: self.metrics.clone(),
        }
    }
}

/// RAII guard for an engine acquired from the pool
pub struct PooledEngine {
    pub engine_mutex: Arc<Mutex<Arc<EngineWrapper>>>,
    _permit: tokio::sync::OwnedSemaphorePermit,
    pool: EnginePool,
    index: usize,
}

impl Drop for PooledEngine {
    fn drop(&mut self) {
        self.pool
            .metrics
            .active_engines
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        debug!("Engine {} returned to pool", self.index);
    }
}

/// Statistics about the engine pool
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub pool_size: usize,
    pub active_engines: usize,
}

impl PoolStats {
    pub fn utilization_percent(&self) -> f64 {
        (self.active_engines as f64 / self.pool_size as f64) * 100.0
    }
}
