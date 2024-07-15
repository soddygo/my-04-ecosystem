use anyhow::Result;
use axum::routing::get;
use axum::Router;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::{RandomIdGenerator, Sampler, Tracer};
use opentelemetry_sdk::{trace, Resource};
use std::time::Duration;
use tonic::metadata::{MetadataMap, MetadataValue};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, Layer};

#[tokio::main]
async fn main() -> Result<()> {
    let tracer = init_trace()?;
    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let file_appender = tracing_appender::rolling::daily(
        "/Users/soddy/Documents/rust_git_workspace/my-04-ecosystem/logs",
        "ecosystem.logs",
    );
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let file = fmt::Layer::new()
        .with_writer(non_blocking)
        .pretty()
        .with_target(true)
        .with_thread_names(true)
        .with_filter(LevelFilter::INFO);

    // console layer for tracing-subscriber
    let console = fmt::Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        .pretty()
        .with_filter(LevelFilter::DEBUG);

    tracing_subscriber::registry()
        .with(opentelemetry)
        .with(console)
        .with(file)
        .init();

    // build our application with a single route
    let shorten_router=init_shorten_router()?;
    let app = Router::new()
        .merge(shorten_router)
        .route("/", get(|| async { "Hello, World!" }));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await?;
    Ok(())
}

///init shorten router
fn init_shorten_router()->Result<Router>{

    let shorten_router= Router::new()
        .route("/:id", get(|| async { "Hello, World! id" }))
        .route("/create",get(|| async { "Hello, World! create" }))
        ;

    Ok(shorten_router)
}

/// opentelemetry-otlp tracer init
fn init_trace() -> Result<Tracer> {
    let mut map = MetadataMap::with_capacity(3);

    map.insert("x-host", "example.com".parse().unwrap());
    map.insert("x-number", "123".parse().unwrap());
    map.insert_bin(
        "trace-proto-bin",
        MetadataValue::from_bytes(b"[binary data]"),
    );

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317")
                .with_timeout(Duration::from_secs(3)), // .with_metadata(map)
        )
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_id_generator(RandomIdGenerator::default())
                .with_max_events_per_span(64)
                .with_max_attributes_per_span(16)
                .with_max_events_per_span(16)
                .with_resource(Resource::new(vec![KeyValue::new(
                    "service.name",
                    "my_axum",
                )])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    Ok(tracer)
}
