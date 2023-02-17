use crate::error::{InitError, ShutdownError};
use opentelemetry::{
    global::GlobalMeterProvider,
    metrics::noop::NoopMeterProvider,
    sdk::{
        export::metrics::aggregation,
        metrics::{controllers, processors, selectors},
    },
};

pub fn init() -> Result<(), InitError> {
    let controller = controllers::basic(
        processors::factory(
            selectors::simple::histogram([1_f64, 2_f64, 5_f64, 10_f64, 20_f64, 50_f64]),
            aggregation::cumulative_temporality_selector(),
        )
        .with_memory(true),
    )
    .build();
    opentelemetry_prometheus::exporter(controller).init();

    Ok(())
}

pub fn shutdown() -> Result<(), ShutdownError> {
    opentelemetry::global::set_meter_provider(GlobalMeterProvider::new(NoopMeterProvider::new()));
    Ok(())
}
