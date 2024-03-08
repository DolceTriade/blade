use std::sync::Mutex;

use anyhow::Context;
use lazy_static::lazy_static;
use prometheus_client::{encoding::text::encode, registry::Metric};

lazy_static! {
    static ref REGISTRY: Mutex<prometheus_client::registry::Registry> =
        Mutex::new(prometheus_client::registry::Registry::default());
}

pub fn register_metric<N, H, M>(name: N, help: H, metric: M) -> M
where
    N: Into<String>,
    H: Into<String>,
    M: Metric + Clone,
{
    let mut r = REGISTRY.lock().unwrap();
    r.register(name, help, metric.clone());
    metric
}

pub fn openmetrics_string() -> anyhow::Result<String> {
    let mut ret: String = "".to_string();
    let r = REGISTRY.lock().unwrap();
    encode(&mut ret, &r)
        .map(|_| ret)
        .context("failed to generate metrics")
}

#[cfg(test)]
mod test {
    use crate::{openmetrics_string, register_metric};

    #[test]
    fn test_register() {
        let c = register_metric(
            "metric",
            "help",
            prometheus_client::metrics::counter::Counter::<u64>::default(),
        );
        let enc1 = openmetrics_string().unwrap();
        assert!(enc1.contains("metric_total 0"));
        c.inc();
        let enc2 = openmetrics_string().unwrap();
        assert!(enc2.contains("metric_total 1"));
    }
}
