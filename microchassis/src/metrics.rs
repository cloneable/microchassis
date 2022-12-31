use crate::error::{InitError, ShutdownError};
use opentelemetry::{
    global::GlobalMeterProvider,
    metrics::noop::NoopMeterProvider,
    sdk::{
        export::metrics::aggregation,
        metrics::{controllers, processors, selectors},
    },
};

pub(crate) fn init() -> Result<(), InitError> {
    let controller = controllers::basic(
        processors::factory(
            selectors::simple::histogram([1.0, 2.0, 5.0, 10.0, 20.0, 50.0]),
            aggregation::cumulative_temporality_selector(),
        )
        .with_memory(true),
    )
    .build();
    opentelemetry_prometheus::exporter(controller).init();

    Ok(())
}

pub(crate) fn shutdown() -> Result<(), ShutdownError> {
    opentelemetry::global::set_meter_provider(GlobalMeterProvider::new(NoopMeterProvider::new()));
    Ok(())
}
