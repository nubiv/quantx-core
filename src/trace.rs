use derive_more::Constructor;
use tracing::subscriber;
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
    pub fn with_filter_level(mut self, level: TracingFilterLevel) -> Self {
        self.filter_level = Some(level);
        self
    }

    pub fn with_format(mut self, format: TracingFormat) -> Self {
        self.format = Some(format);
        self
    }

    pub fn with_timer(mut self, timer: TracingTimer) -> Self {
        self.timer = Some(timer);
        self
    }

    pub fn with_file_persistence(mut self, persistence: TracingFilePersistence) -> Self {
        self.file_persistence = Some(persistence);
        self
    }

    pub fn init(mut self) {
        let file_persistence = self.file_persistence.take();

        if let Some(TracingFilePersistence::Enabled(config)) = file_persistence {
            self.init_with_persistence(config);
        } else {
            self.init_with_stdout();
        }
    }

    fn init_with_stdout(self) {
        let sub_builder = tracing_subscriber::FmtSubscriber::builder();
        let subscriber = sub_builder.finish();

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
        let subscriber = subscriber.with(filter_layer);

        let timer = self.timer.unwrap_or_default();
        match timer {
            TracingTimer::Default => {
                // Default timer is already set in FmtSubscriber
            },
            TracingTimer::Local => {
                let local_timer = ChronoLocalTimer;
                let timer_layer = tracing_subscriber::fmt::layer().with_timer(local_timer);
                let subscriber = subscriber.with(timer_layer);
                return subscriber.init();
            },
        }
        // tracing_subscriber::layer::SubscriberExt::with(subscriber, layer);
        // subscriber.with(tracing_subscriber::layer::SubscriberExt::with_timer());

        match format {
            TracingFormat::Plain => {
                subscriber.with(tracing_subscriber::fmt::layer()).init();
            },
            TracingFormat::Json => {
                subscriber.with(tracing_subscriber::fmt::layer().json().flatten_event(true)).init();
            },
        };
    }

    fn init_with_persistence(self, config: TracingFilePersistenceConfig) {
        let sub_builder = tracing_subscriber::FmtSubscriber::builder();

        let log_dir = config.log_dir;
        let log_prefix = config.log_prefix;
        let file_appender = match config.rotation {
            TracingFileRotation::Daily => tracing_appender::rolling::daily(log_dir, log_prefix),
            TracingFileRotation::Hourly => tracing_appender::rolling::hourly(log_dir, log_prefix),
            TracingFileRotation::Minutely => tracing_appender::rolling::minutely(log_dir, log_prefix),
            TracingFileRotation::Never => tracing_appender::rolling::never(log_dir, log_prefix),
        };
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

        let subscriber = sub_builder.with_writer(non_blocking).finish();

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
        let subscriber = subscriber.with(filter_layer);

        match format {
            TracingFormat::Plain => {
                subscriber.with(tracing_subscriber::fmt::layer()).init();
            },
            TracingFormat::Json => {
                subscriber.with(tracing_subscriber::fmt::layer().json().flatten_event(true)).init();
            },
        };
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

pub fn init_tracing_default() {
    let log_dir = "./log";
    let log_prefix = "log";
    let file_appender = tracing_appender::rolling::hourly(log_dir, log_prefix);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let filter_layer = tracing_subscriber::filter::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env_lossy();
    let fmt_layer = tracing_subscriber::fmt::layer().json().flatten_event(true);

    tracing_subscriber::FmtSubscriber::builder()
        .with_writer(non_blocking)
        .with_timer(ChronoLocalTimer)
        .finish()
        .with(filter_layer)
        .with(fmt_layer)
        .init()
}

pub fn init_tracing() {
    let log_dir = "./log";
    let log_prefix = "log";
    let file_appender = tracing_appender::rolling::hourly(log_dir, log_prefix);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let filter_layer = tracing_subscriber::filter::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env_lossy();
    let fmt_layer = tracing_subscriber::fmt::layer().json().flatten_event(true);

    tracing_subscriber::FmtSubscriber::builder()
        .with_writer(non_blocking)
        .with_timer(ChronoLocalTimer)
        .finish()
        .with(filter_layer)
        .with(fmt_layer)
        .init()
}

#[allow(dead_code)]
struct ChronoLocalTimer;

impl tracing_subscriber::fmt::time::FormatTime for ChronoLocalTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        write!(w, "{}", chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.f%:z"))
    }
}

// pub fn init_tracing() {
//     tracing_subscriber::registry()
//         .with(
//             tracing_subscriber::filter::EnvFilter::builder()
//                 .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
//                 .from_env_lossy(),
//         )
//         .with(tracing_subscriber::fmt::layer())
//         .init()
// }

pub fn init_json_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::filter::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer().json().flatten_event(true))
        .init()
}
