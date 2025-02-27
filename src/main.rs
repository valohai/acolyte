use tracing::{info, warn};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

fn main() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .with(sentry_tracing::layer())
        .init();

    let acolyte_id = Uuid::new_v4();
    let sentry_guard = init_sentry(&acolyte_id);
    if sentry_guard.is_some() {
        info!("Sentry initialized");
    } else {
        warn!("Sentry NOT initialized");
    }

    loop {
        // imitate work...
        info!("Acolyte: For Ner'zhul!");
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn init_sentry(acolyte_id: &Uuid) -> Option<sentry::ClientInitGuard> {
    let dsn = std::env::var("SENTRY_DSN").ok()?;
    let release = sentry::release_name!();
    let guard = sentry::init((
        dsn,
        sentry::ClientOptions {
            release,
            ..Default::default()
        },
    ));

    sentry::configure_scope(|scope| {
        scope.set_tag("acolyte_id", acolyte_id);
        let cluster_name = std::env::var("CLUSTER_NAME").unwrap_or("Unknown".to_string());
        scope.set_tag("cluster.name", cluster_name);
    });

    Some(guard)
}
