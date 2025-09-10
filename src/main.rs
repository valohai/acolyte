use acolyte::config::Config;
use acolyte::consts::{ID_ENV_VAR, MAX_RUN_ATTEMPTS, RESTART_DELAY_SECS};
use libc::{SIG_IGN, SIGHUP};
use std::time::Duration;
use std::{env, os::unix::process::CommandExt, panic, process, thread};
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

const NO_RESTART_ENV_VAR: &str = "ACOLYTE_NO_RESTART";
const RESTART_ENV_VAR: &str = "ACOLYTE_RESTART";
fn is_no_restart() -> bool {
    env::var(NO_RESTART_ENV_VAR)
        .map(|v| !v.is_empty())
        .unwrap_or(false)
}

fn get_restart_count() -> u8 {
    env::var(RESTART_ENV_VAR)
        .ok()
        .and_then(|count_str| count_str.parse::<u8>().ok())
        .unwrap_or(0)
}

fn main() {
    nohup();

    init_logging();

    let config = Config::from_env();
    let sentry_guard = init_sentry(&config);
    if sentry_guard.is_some() {
        info!("Sentry initialized");
    } else {
        warn!("Sentry NOT initialized");
    }

    if is_no_restart() {
        info!("No-restart mode enabled; running Acolyte without restart logic");
        acolyte::run_acolyte(&config);
        process::exit(0);
    } else {
        run_with_restart(&config);
    }
}

fn run_with_restart(config: &Config) {
    let restart_count = get_restart_count();
    if restart_count > 0 {
        info!(
            "Restarting Acolyte - waiting {} seconds (attempt {}/{})",
            RESTART_DELAY_SECS,
            restart_count + 1,
            MAX_RUN_ATTEMPTS
        );
        thread::sleep(Duration::from_secs(RESTART_DELAY_SECS));
    }

    info!(
        "Starting Acolyte {} (attempt {}/{})",
        config.acolyte_id,
        restart_count + 1,
        MAX_RUN_ATTEMPTS
    );

    let run_result = panic::catch_unwind(|| acolyte::run_acolyte(config));
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
        let current_exe = std::env::current_exe().expect("Failed to get current executable path");
        let err = process::Command::new(current_exe)
            .env(RESTART_ENV_VAR, next_count.to_string())
            .env(ID_ENV_VAR, config.acolyte_id.to_string())
            .args(std::env::args().skip(1))
            .exec();

        // exec failed, wouldn't reach here otherwise
        error!("Failed to restart Acolyte: {}", err);
        process::exit(1);
    }
}

fn nohup() {
    // Replicate what `nohup` command does; ignore SIGHUP (signal sent when terminal disconnects)
    unsafe {
        libc::signal(SIGHUP, SIG_IGN);
    }
}

fn init_logging() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .with(sentry_tracing::layer())
        .init();
}

fn init_sentry(config: &Config) -> Option<sentry::ClientInitGuard> {
    let release = sentry::release_name!();
    let dsn = config.sentry_dsn.clone();
    let guard = sentry::init((
        dsn,
        sentry::ClientOptions {
            release,
            ..Default::default()
        },
    ));

    sentry::configure_scope(|scope| {
        scope.set_tag("acolyte_id", config.acolyte_id);
        scope.set_tag("cluster.name", &config.cluster_name);
    });

    Some(guard)
}
