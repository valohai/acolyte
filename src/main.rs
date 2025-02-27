use std::time::Duration;
use std::{env, os::unix::process::CommandExt, panic, process, thread};
use tracing::{error, info, warn};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

const MAX_RUN_ATTEMPTS: u8 = 5;
const RESTART_DELAY_SECS: u64 = 10;
const RESTART_ENV_VAR: &str = "ACOLYTE_RESTART";
const ACOLYTE_ID_ENV_VAR: &str = "ACOLYTE_ID";

fn main() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .with(sentry_tracing::layer())
        .init();

    let acolyte_id = match env::var(ACOLYTE_ID_ENV_VAR) {
        Ok(id) => match Uuid::parse_str(&id) {
            Ok(uuid) => {
                info!("Using existing Acolyte ID from environment");
                uuid
            }
            Err(_) => {
                warn!("Invalid Acolyte ID format in environment, generating a new one");
                Uuid::new_v4()
            }
        },
        Err(_) => Uuid::new_v4(),
    };

    let sentry_guard = init_sentry(&acolyte_id);
    if sentry_guard.is_some() {
        info!("Sentry initialized");
    } else {
        warn!("Sentry NOT initialized");
    }

    let restart_count = match env::var(RESTART_ENV_VAR) {
        Ok(count_str) => {
            let count = count_str.parse::<u8>().unwrap_or(0);
            info!(
                "Restarting Acolyte - waiting {} seconds (attempt {}/{})",
                RESTART_DELAY_SECS,
                count + 1,
                MAX_RUN_ATTEMPTS
            );
            thread::sleep(Duration::from_secs(RESTART_DELAY_SECS));
            count
        }
        Err(_) => 0,
    };

    info!(
        "Starting Acolyte {} (attempt {}/{})",
        acolyte_id,
        restart_count + 1,
        MAX_RUN_ATTEMPTS
    );

    let run_result = panic::catch_unwind(|| acolyte::run_acolyte());
    if run_result.is_ok() {
        process::exit(0);
    } else {
        let next_count = restart_count + 1;

        if next_count >= MAX_RUN_ATTEMPTS {
            error!(
                "Maximum run attempts ({}) reached after crash. Exiting.",
                MAX_RUN_ATTEMPTS
            );
            process::exit(1);
        }

        warn!("Acolyte crashed. Executing new process...");
        let current_exe = env::current_exe().expect("Failed to get current executable path");
        let err = process::Command::new(current_exe)
            .env(RESTART_ENV_VAR, next_count.to_string())
            .env(ACOLYTE_ID_ENV_VAR, acolyte_id.to_string())
            .args(env::args().skip(1))
            .exec();

        // exec failed, wouldn't reach here otherwise
        error!("Failed to restart Acolyte: {}", err);
        process::exit(1);
    }
}

fn init_sentry(acolyte_id: &Uuid) -> Option<sentry::ClientInitGuard> {
    let dsn = env::var("SENTRY_DSN").ok()?;
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
        let cluster_name = env::var("CLUSTER_NAME").unwrap_or("Unknown".to_string());
        scope.set_tag("cluster.name", cluster_name);
    });

    Some(guard)
}
