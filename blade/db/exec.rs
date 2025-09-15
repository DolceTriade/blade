use std::sync::Arc;
use anyhow::Result;
use lazy_static::lazy_static;
use prometheus_client::metrics::{counter::Counter, gauge::Gauge, histogram::Histogram};
use std::sync::atomic::AtomicU32;
use tracing::instrument;

lazy_static! {
    static ref DB_EXEC_TOTAL: Counter::<u64> = metrics::register_metric(
        "blade_db_exec_total",
        "Total number of DB executions via spawn_blocking",
        Counter::default()
    );
    static ref DB_EXEC_GROUPED_TOTAL: Counter::<u64> = metrics::register_metric(
        "blade_db_exec_grouped_total",
        "Total number of grouped DB executions",
        Counter::default()
    );
    static ref DB_BLOCKING_INFLIGHT: Gauge::<u32, AtomicU32> = metrics::register_metric(
        "blade_db_blocking_inflight",
        "Number of DB operations currently running in spawn_blocking",
        Gauge::default()
    );
    static ref DB_EXEC_DURATION: Histogram = {
        let buckets = [0.001, 0.005, 0.010, 0.025, 0.050, 0.100, 0.250, 0.500, 1.0, 2.5, 5.0, 10.0];
        metrics::register_metric(
            "blade_db_exec_duration_seconds",
            "Duration of DB operations in spawn_blocking",
            Histogram::new(buckets.into_iter())
        )
    };
}

/// Execute a single database operation in a blocking task.
/// This is the basic helper for migrating sync DB calls to async contexts.
#[instrument(skip(mgr, f), name = "db_exec")]
pub async fn run<T, F>(mgr: Arc<dyn state::DBManager>, f: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce(&mut dyn state::DB) -> Result<T> + Send + 'static,
{
    DB_EXEC_TOTAL.inc();
    DB_BLOCKING_INFLIGHT.inc();
    let start = std::time::Instant::now();

    let result = tokio::task::spawn_blocking(move || {
        let mut db = mgr.get()?;
        f(db.as_mut())
    })
    .await
    .map_err(|e| anyhow::anyhow!("DB task join error: {e}"))?;

    DB_BLOCKING_INFLIGHT.dec();
    DB_EXEC_DURATION.observe(start.elapsed().as_secs_f64());
    result
}

/// Execute multiple database operations in a single blocking task.
/// This amortizes the spawn_blocking overhead for related operations.
#[instrument(skip(mgr, f), name = "db_exec_group")]
pub async fn run_group<T, F>(mgr: Arc<dyn state::DBManager>, f: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce(&mut dyn state::DB) -> Result<T> + Send + 'static,
{
    DB_EXEC_GROUPED_TOTAL.inc();
    DB_BLOCKING_INFLIGHT.inc();
    let start = std::time::Instant::now();

    let result = tokio::task::spawn_blocking(move || {
        let mut db = mgr.get()?;
        f(db.as_mut())
    })
    .await
    .map_err(|e| anyhow::anyhow!("DB task join error: {e}"))?;

    DB_BLOCKING_INFLIGHT.dec();
    DB_EXEC_DURATION.observe(start.elapsed().as_secs_f64());
    result
}

/// Execute database operations within a transaction.
/// The closure must be synchronous and perform all DB operations sequentially.
#[instrument(skip(mgr, f), name = "db_transaction")]
pub async fn transaction<T, F>(mgr: Arc<dyn state::DBManager>, f: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce(&mut dyn state::DB) -> Result<T> + Send + 'static,
{
    // For now, this is identical to run_group but provides semantic clarity
    // Later we could add actual transaction handling if needed
    run_group(mgr, f).await
}
