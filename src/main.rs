use acolyte::{consts, env};
use libc::{SIG_IGN, SIGHUP};
use std::time::Duration;
use std::{os::unix::process::CommandExt, panic, process, thread};
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;
use uuid::Uuid;

fn main() {
    nohup();

    init_logging();

    let acolyte_id = env::get_or_create_acolyte_id();
    let sentry_guard = init_sentry(&acolyte_id);
    if sentry_guard.is_some() {
        info!("Sentry initialized");
    } else {
        warn!("Sentry NOT initialized");
    }

    let restart_count = env::get_restart_count();
    if restart_count > 0 {
        info!(
            "Restarting Acolyte - waiting {} seconds (attempt {}/{})",
            consts::RESTART_DELAY_SECS,
            restart_count + 1,
            consts::MAX_RUN_ATTEMPTS
        );
        thread::sleep(Duration::from_secs(consts::RESTART_DELAY_SECS));
    }

    info!(
        "Starting Acolyte {} (attempt {}/{})",
        acolyte_id,
        restart_count + 1,
        consts::MAX_RUN_ATTEMPTS
    );

    let run_result = panic::catch_unwind(acolyte::run_acolyte);
    if run_result.is_ok() {
        process::exit(0);
    } else {
        let next_count = restart_count + 1;

        if next_count >= consts::MAX_RUN_ATTEMPTS {
            error!(
                "Maximum run attempts ({}) reached after crash. Exiting.",
                consts::MAX_RUN_ATTEMPTS
            );
            process::exit(1);
        }

        warn!("Acolyte crashed. Executing new process...");
        let current_exe = std::env::current_exe().expect("Failed to get current executable path");
        let err = process::Command::new(current_exe)
            .env(env::RESTART_ENV_VAR, next_count.to_string())
            .env(env::ID_ENV_VAR, acolyte_id.to_string())
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

fn init_sentry(acolyte_id: &Uuid) -> Option<sentry::ClientInitGuard> {
    let dsn = env::get_sentry_dsn()?;
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
        let cluster_name = env::get_cluster_name();
        scope.set_tag("cluster.name", cluster_name);
    });

    Some(guard)
}
