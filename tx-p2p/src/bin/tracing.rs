use opentelemetry::global;
use opentelemetry_sdk::propagation::{
    BaggagePropagator, TextMapCompositePropagator, TraceContextPropagator,
};
use snafu::ResultExt;
use tracing_subscriber::{
    filter::EnvFilter,
    layer::{Layered, SubscriberExt},
    reload,
    util::SubscriberInitExt,
    Layer, Registry,
};

use crate::error;

pub type FilterHandle =
    reload::Handle<EnvFilter, Layered<Box<dyn Layer<Registry> + Send + Sync>, Registry>>;

pub fn init_tracing<S>(default_filter_directives: S) -> error::Result<FilterHandle>
where
    S: AsRef<str>,
{
    global::set_text_map_propagator(TextMapCompositePropagator::new(vec![
        Box::new(BaggagePropagator::new()),
        Box::new(TraceContextPropagator::new()),
    ]));

    // filter
    let (filter_layer, handle) = reload::Layer::new(
        EnvFilter::builder()
            .try_from_env()
            .or_else(|_| EnvFilter::builder().parse(default_filter_directives))
            .context(error::ParseFilteringDirectivesSnafu)?,
    );

    // format
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_thread_ids(true)
        .with_thread_names(true)
        .pretty()
        .boxed();

    // subscriber
    tracing_subscriber::registry().with(fmt_layer).with(filter_layer).init();

    Ok(handle)
}
