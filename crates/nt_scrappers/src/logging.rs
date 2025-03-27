use tracing::Level;
use std::sync::Once;
use std::collections::VecDeque;

static INIT: Once = Once::new();

pub struct Logger {
    prefixes: VecDeque<String>,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            prefixes: VecDeque::new(),
        }
    }

    pub fn with_new_prefixes(mut self, prefix: String) -> Self {
        self.prefixes.clear();
        self.prefixes.push_back(prefix);
        self
    }

    pub fn with_prefix(mut self, prefix: String) -> Self {
        self.prefixes.push_back(prefix);
        self
    }

    pub fn info(&self, message: &str) {
        let prefix = self.prefixes.iter().map(|p| format!("{} ", p)).collect::<String>();
        tracing::info!("{}{}", prefix, message);
    }

    pub fn error(&self, message: &str) {
        let prefix = self.prefixes.iter().map(|p| format!("{} ", p)).collect::<String>();
        tracing::error!("{}{}", prefix, message);
    }

    pub fn warn(&self, message: &str) {
        let prefix = self.prefixes.iter().map(|p| format!("{} ", p)).collect::<String>();
        tracing::warn!("{}{}", prefix, message);
    }

    pub fn debug(&self, message: &str) {
        let prefix = self.prefixes.iter().map(|p| format!("{} ", p)).collect::<String>();
        tracing::debug!("{}{}", prefix, message);
    }
}

pub fn init_logging() -> Logger {
    if !tracing::dispatcher::has_been_set() {
        INIT.call_once(|| {
            tracing_subscriber::fmt()
                .with_max_level(Level::INFO)
                .init();
        });
    }
    Logger::new()
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*)
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        tracing::error!($($arg)*)
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*)
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*)
    };
} 