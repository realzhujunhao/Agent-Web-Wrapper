/// Log configuration.  
///
/// ### Semantics of each level  
/// - Error: Server cannot keep up, must abort in place (server scope)  
/// - Warn: Client request failed (user scope)  
/// - Info: Server states  
/// - Debug: *  
use std::{env, fs::create_dir_all};

use time::macros::{format_description, offset};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    Layer, fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt,
};

/// Indoc format flavor of tracing error, abort!
#[macro_export]
macro_rules! indoc_error {
    ($($t:tt)*) => {{
        let content = indoc::formatdoc!($($t)*);
        ::tracing::error!("{}", content);
        std::process::exit(1)
    }};
}

/// Indoc format flavor of tracing warn
#[macro_export]
macro_rules! indoc_warn {
    ($($t:tt)*) => {{
        let content = indoc::formatdoc!($($t)*);
        ::tracing::warn!("{}", content);
    }};
}

/// Indoc format flavor of tracing info
#[macro_export]
macro_rules! indoc_info {
    ($($t:tt)*) => {{
        let content = indoc::formatdoc!($($t)*);
        ::tracing::info!("{}", content);
    }};
}

/// Indoc format flavor of tracing debug
#[macro_export]
macro_rules! indoc_debug {
    ($($t:tt)*) => {{
        let content = indoc::formatdoc!($($t)*);
        ::tracing::debug!("{}", content);
    }};
}

pub fn init_tracing(debug: bool) -> tracing_appender::non_blocking::WorkerGuard {
    let filter = if debug {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };

    let exec_path = env::current_exe().expect("cannot get exec path");
    let exec_path = exec_path
        .canonicalize()
        .expect("convert exec path to absolute path");
    let exec_dir = exec_path.parent().expect("exec has no parent");
    let log_dir = exec_dir.join("server_log");
    create_dir_all(&log_dir).expect("create log dir");
    let offset = offset!(+8);
    let formatter = format_description!("[year]/[month]/[day]-[hour]:[minute]:[second]");
    let time = tracing_subscriber::fmt::time::OffsetTime::new(offset, formatter);
    let file_appender = tracing_appender::rolling::daily(log_dir, "");
    let (non_block_file_wt, guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_timer(time.clone())
        .with_line_number(true)
        .with_thread_ids(true)
        .with_span_events(FmtSpan::ACTIVE)
        .with_writer(non_block_file_wt)
        .with_ansi(false)
        .with_target(false)
        .with_filter(LevelFilter::DEBUG);
    let std_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_timer(time)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_span_events(FmtSpan::ACTIVE)
        .with_target(false)
        .with_filter(filter);

    tracing_subscriber::registry()
        .with(file_layer)
        .with(std_layer)
        .init();
    guard
}
