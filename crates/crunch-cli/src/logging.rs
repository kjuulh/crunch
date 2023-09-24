use clap::ValueEnum;
use tracing::Level;

#[derive(Clone, ValueEnum)]
pub enum LogArg {
    None,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogArg {
    pub fn init_logging(&self) {
        match self {
            LogArg::None => {}
            LogArg::Trace => {
                tracing_subscriber::fmt()
                    .with_max_level(Level::TRACE)
                    .init();
            }
            LogArg::Debug => {
                tracing_subscriber::fmt()
                    .with_max_level(Level::DEBUG)
                    .init();
            }
            LogArg::Info => {
                tracing_subscriber::fmt().with_max_level(Level::INFO).init();
            }
            LogArg::Warn => {
                tracing_subscriber::fmt().with_max_level(Level::WARN).init();
            }
            LogArg::Error => {
                tracing_subscriber::fmt()
                    .with_max_level(Level::ERROR)
                    .init();
            }
        }
    }
}
