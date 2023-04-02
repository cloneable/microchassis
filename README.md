# microchassis

[Work in Progress]

The microchassis is all about increasing the observability of Rust binaries.

## Memory Profiling with jemalloc

Example intergration with `hyper` endpoint:

```rust
use microchassis::profiling::jeprof;
```

```rust
    let symbol_table = Arc::new(jeprof::SymbolTable::load()?);
    let make_service = hyper::service::make_service_fn(move |_conn| {
        let symbol_table = Arc::clone(&symbol_table);
        let service = hyper::service::service_fn(move |req| handle(Arc::clone(&symbol_table), req));
        async move { Ok::<_, io::Error>(service) }
    });
    let addr = SocketAddr::from(([127, 0, 0, 1], 12345));
    tokio::spawn(async move { hyper::Server::bind(&addr).serve(make_service).await });
```

```rust
async fn handle(
    symbol_table: Arc<jeprof::SymbolTable>,
    req: hyper::Request<hyper::Body>,
) -> io::Result<hyper::Response<hyper::Body>> {
    let (parts, body) = req.into_parts();
    let body = hyper::body::to_bytes(body).await?;
    let req = hyper::Request::from_parts(parts, body.into());

    let resp = jeprof::router(symbol_table.as_ref(), req)?;

    let (parts, body) = resp.into_parts();
    let body = hyper::Body::from(body);
    let resp = hyper::Response::from_parts(parts, body);

    Ok(resp)
}
```

Keep symbol in release binary.

```toml
[profile.release]
debug = 1
strip = false
```

Disable ASLR if necessary or install the `disable_aslr` helper tool.

```shell
cargo install microchassis
```

Use package manager to install `jeprof` or download from
<https://github.com/jemalloc/jemalloc/blob/master/bin/jeprof.in>, rename to
`jeprof` and edit its version (x.y.z-0-commit) inside.

(optional) Install `graphviz` package.

(optional) Install `flamegraph.pl` from
<https://raw.githubusercontent.com/brendangregg/FlameGraph/master/flamegraph.pl>.

Run your binary with profiling enabled and activated:

```shell
MALLOC_CONF=prof:true,prof_active:true,lg_prof_sample:8 disable_aslr ./myserver
```

(Use `_RJEM_MALLOC_CONF` if jemalloc is built with prefix.)

Or start with profiling not active and activate later via HTTP endpoint:

```shell
MALLOC_CONF=prof:true,prof_active:false,lg_prof_sample:8 disable_aslr ./myserver
```

```shell
curl -X POST 'http://myserver:12345/pprof/conf?prof.active:true'
```

Fetch a profile dump with `jeprof` and generate a flame/icicle graph.

```shell
jeprof --raw 'http://myserver:12345/pprof/heap' >heap.prof
jeprof --collapsed heap.prof | flamegraph.pl --reverse --invert >heap.svg
```
