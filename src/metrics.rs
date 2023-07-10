use std::sync::Arc;

use poem::web::Data;
use prometheus::{IntCounter, IntCounterVec, Opts, Registry, TextEncoder};

pub type MetricsData<'a> = Data<&'a Arc<Metrics>>;

pub struct Metrics {
    registry: Registry,
    pub requests: Requests,
    pub cache_hits: CacheHits,
}

pub struct Requests {
    pub config: IntCounter,
    pub environments: IntCounter,
    pub resource_usage: IntCounterVec,
    pub build_run: IntCounterVec,
    pub build: IntCounterVec,
    pub run: IntCounter,
}

pub struct CacheHits {
    pub resource_usage: IntCounterVec,
    pub build_run: IntCounterVec,
    pub build: IntCounterVec,
}

impl Metrics {
    pub fn new() -> prometheus::Result<Self> {
        let registry = Registry::new_custom(Some("sandkasten".into()), None)?;

        Ok(Self {
            requests: Requests::new(&registry)?,
            cache_hits: CacheHits::new(&registry)?,
            registry,
        })
    }
}

impl Requests {
    fn new(registry: &Registry) -> prometheus::Result<Self> {
        let config = IntCounter::new("config_requests", "Number of config requests")?;
        let environments =
            IntCounter::new("environments_requests", "Number of environments requests")?;
        let resource_usage = IntCounterVec::new(
            Opts::new(
                "resource_usage_requests",
                "Number of resource_usage requests",
            ),
            &["environment"],
        )?;
        let build_run = IntCounterVec::new(
            Opts::new("build_run_requests", "Number of build_run requests"),
            &["environment"],
        )?;
        let build = IntCounterVec::new(
            Opts::new("build_requests", "Number of build requests"),
            &["environment"],
        )?;
        let run = IntCounter::new("run_requests", "Number of run requests")?;
        registry.register(Box::new(config.clone()))?;
        registry.register(Box::new(environments.clone()))?;
        registry.register(Box::new(resource_usage.clone()))?;
        registry.register(Box::new(build_run.clone()))?;
        registry.register(Box::new(build.clone()))?;
        registry.register(Box::new(run.clone()))?;

        Ok(Self {
            config,
            environments,
            resource_usage,
            build_run,
            build,
            run,
        })
    }
}

impl CacheHits {
    fn new(registry: &Registry) -> prometheus::Result<Self> {
        let resource_usage = IntCounterVec::new(
            Opts::new(
                "resource_usage_cache_hits",
                "Number of cache hits in resource_usage",
            ),
            &["environment"],
        )?;
        let build_run = IntCounterVec::new(
            Opts::new("build_run_cache_hits", "Number of cache hits in build_run"),
            &["environment"],
        )?;
        let build = IntCounterVec::new(
            Opts::new("build_cache_hits", "Number of cache hits in build"),
            &["environment"],
        )?;
        registry.register(Box::new(resource_usage.clone()))?;
        registry.register(Box::new(build_run.clone()))?;
        registry.register(Box::new(build.clone()))?;

        Ok(Self {
            resource_usage,
            build_run,
            build,
        })
    }
}

#[poem::handler]
pub fn endpoint(metrics: Data<&Arc<Metrics>>) -> anyhow::Result<String> {
    let encoder = TextEncoder::new();
    let metric_families = metrics.0.registry.gather();
    Ok(encoder.encode_to_string(&metric_families)?)
}
