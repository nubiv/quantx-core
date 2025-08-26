use derive_more::Constructor;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const DEFAULT_LOG_DIR: &str = "logs";
const DEFAULT_LOG_PREFIX: &str = "log";

#[derive(Debug, Default, Constructor)]
pub struct TracingSubscriber {
    filter_level: Option<TracingFilterLevel>,
    format: Option<TracingFormat>,
    timer: Option<TracingTimer>,
    file_persistence: Option<TracingFilePersistence>,
}

impl TracingSubscriber {
    pub fn with_filter_level(&mut self, level: TracingFilterLevel) {
        self.filter_level = Some(level);
    }

    pub fn with_format(&mut self, format: TracingFormat) {
        self.format = Some(format);
    }

    pub fn with_timer(&mut self, timer: TracingTimer) {
        self.timer = Some(timer);
    }

    pub fn with_file_persistence(&mut self, persistence: TracingFilePersistence) {
        self.file_persistence = Some(persistence);
    }

    pub fn init(mut self) -> Option<tracing_appender::non_blocking::WorkerGuard> {
        let file_persistence = self.file_persistence.take();

        if let Some(TracingFilePersistence::Enabled(config)) = file_persistence {
            Some(self.init_with_persistence(config))
        } else {
            self.init_with_stdout();
            None
        }
    }

    fn init_with_stdout(self) {
        let timer = self.timer.unwrap_or_default();
        let filter_level = self.filter_level.unwrap_or_default();
        let format = self.format.unwrap_or_default();

        let subscriber = tracing_subscriber::registry();

        let filter_layer_builder = tracing_subscriber::filter::EnvFilter::builder();
        let filter_level = match filter_level {
            TracingFilterLevel::Error => tracing_subscriber::filter::LevelFilter::ERROR,
            TracingFilterLevel::Warn => tracing_subscriber::filter::LevelFilter::WARN,
            TracingFilterLevel::Info => tracing_subscriber::filter::LevelFilter::INFO,
            TracingFilterLevel::Debug => tracing_subscriber::filter::LevelFilter::DEBUG,
            TracingFilterLevel::Trace => tracing_subscriber::filter::LevelFilter::TRACE,
        };
        let filter_layer = filter_layer_builder.with_default_directive(filter_level.into()).from_env_lossy();

        match (format, timer) {
            (TracingFormat::Plain, TracingTimer::Default) => {
                subscriber.with(filter_layer).init();
            },
            (TracingFormat::Plain, TracingTimer::Local) => {
                let local_timer = ChronoLocalTimer;
                let fmt_layer = tracing_subscriber::fmt::layer().with_timer(local_timer);

                subscriber.with(filter_layer).with(fmt_layer).init();
            },
            (TracingFormat::Json, TracingTimer::Default) => {
                let fmt_layer = tracing_subscriber::fmt::layer().json().flatten_event(true);
                subscriber.with(filter_layer).with(fmt_layer).init();
            },
            (TracingFormat::Json, TracingTimer::Local) => {
                let local_timer = ChronoLocalTimer;
                let fmt_layer = tracing_subscriber::fmt::layer().with_timer(local_timer).json().flatten_event(true);

                subscriber.with(filter_layer).with(fmt_layer).init();
            },
        };
    }

    fn init_with_persistence(self, config: TracingFilePersistenceConfig) -> tracing_appender::non_blocking::WorkerGuard {
        let log_dir = config.log_dir;
        let log_prefix = config.log_prefix;
        let file_appender = match config.rotation {
            TracingFileRotation::Daily => tracing_appender::rolling::daily(log_dir, log_prefix),
            TracingFileRotation::Hourly => tracing_appender::rolling::hourly(log_dir, log_prefix),
            TracingFileRotation::Minutely => tracing_appender::rolling::minutely(log_dir, log_prefix),
            TracingFileRotation::Never => tracing_appender::rolling::never(log_dir, log_prefix),
        };
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        let timer = self.timer.unwrap_or_default();
        let filter_level = self.filter_level.unwrap_or_default();
        let format = self.format.unwrap_or_default();

        let filter_layer_builder = tracing_subscriber::filter::EnvFilter::builder();
        let filter_level = match filter_level {
            TracingFilterLevel::Error => tracing_subscriber::filter::LevelFilter::ERROR,
            TracingFilterLevel::Warn => tracing_subscriber::filter::LevelFilter::WARN,
            TracingFilterLevel::Info => tracing_subscriber::filter::LevelFilter::INFO,
            TracingFilterLevel::Debug => tracing_subscriber::filter::LevelFilter::DEBUG,
            TracingFilterLevel::Trace => tracing_subscriber::filter::LevelFilter::TRACE,
        };
        let filter_layer = filter_layer_builder.with_default_directive(filter_level.into()).from_env_lossy();

        match (format, timer) {
            (TracingFormat::Plain, TracingTimer::Default) => {
                let sub_builder = tracing_subscriber::FmtSubscriber::builder();
                let subscriber = sub_builder.with_writer(non_blocking).finish();

                subscriber.with(filter_layer).init();
            },
            (TracingFormat::Plain, TracingTimer::Local) => {
                let local_timer = ChronoLocalTimer;

                let sub_builder = tracing_subscriber::FmtSubscriber::builder();
                let subscriber = sub_builder.with_timer(local_timer).with_writer(non_blocking).finish();

                subscriber.with(filter_layer).init();
            },
            (TracingFormat::Json, TracingTimer::Default) => {
                let sub_builder = tracing_subscriber::FmtSubscriber::builder();
                let subscriber = sub_builder.json().flatten_event(true).with_writer(non_blocking).finish();

                subscriber.with(filter_layer).init();
            },
            (TracingFormat::Json, TracingTimer::Local) => {
                let local_timer = ChronoLocalTimer;

                let sub_builder = tracing_subscriber::FmtSubscriber::builder();
                let subscriber = sub_builder
                    .with_timer(local_timer)
                    .json()
                    .flatten_event(true)
                    .with_writer(non_blocking)
                    .finish();

                subscriber.with(filter_layer).init();
            },
        };

        guard
    }
}

#[derive(Debug, Default)]
pub enum TracingFilterLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Default)]
pub enum TracingFormat {
    #[default]
    Plain,
    Json,
}

#[derive(Debug, Default)]
pub enum TracingTimer {
    #[default]
    Default,
    Local,
}

#[derive(Debug, Default)]
pub enum TracingFilePersistence {
    Enabled(TracingFilePersistenceConfig),
    #[default]
    Disabled,
}

#[derive(Debug)]
pub struct TracingFilePersistenceConfig {
    pub log_dir: &'static str,
    pub log_prefix: &'static str,
    pub rotation: TracingFileRotation,
}

impl Default for TracingFilePersistenceConfig {
    fn default() -> Self {
        Self {
            log_dir: DEFAULT_LOG_DIR,
            log_prefix: DEFAULT_LOG_PREFIX,
            rotation: TracingFileRotation::default(),
        }
    }
}

#[derive(Debug, Default)]
pub enum TracingFileRotation {
    Daily,
    #[default]
    Hourly,
    Minutely,
    Never,
}

struct ChronoLocalTimer;

impl tracing_subscriber::fmt::time::FormatTime for ChronoLocalTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        write!(w, "{}", chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.9f %:z"))
    }
}

pub fn init_tracing_default() -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let mut subscriber = TracingSubscriber::default();
    subscriber.with_timer(TracingTimer::Local);

    subscriber.init()
}
