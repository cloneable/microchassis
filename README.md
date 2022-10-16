# microchassis

Chassis for Rust Microservices

## ToDo

- [ ] Support other async runtimes than tokio.
- [ ] Provide `microchassis::init()`.
    - [x] Handle and propagate termination signals.
    - [ ] Set up logging. Decide on logging crate(s) to use/support.
    - [ ] Set up tracing with OTel.
    - [ ] Set up metrics registration and export with prometheus or OTel.
    - [ ] Provide build-dep crate to help with build stamping.
    - [ ] Inject information from build stamping.
    - [ ] Start management http(s) server.
    - [ ] Provide liveness/readiness endpoints.
    - [ ] Provide metrics endpoint.
    - [ ] Add builder as parameter to adjust settings?
- [ ] Provide `microchassis::shutdown()` to explicitly shut down everything.
- [ ] Distinguish between running as service and locally and provide different defaults for logging/tracing/timezone/etc.
- [ ] Logging preamble.
- [ ] tokio console-subscriber for dev env instead of span exporter. (tokio_unstable)
- [ ] config support?
- [ ] Catch and log panics with stacktraces as log entry.
