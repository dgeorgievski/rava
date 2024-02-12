# Tracing

## Spans
```json
{
    "v":0
    name":"healthcat",
    "msg":"[HTTP REQUEST - START]",
    "level":30,
    "hostname":"healthcat-rs-57bc7f7565-gxsf2",
    "pid":1,
    "time":"2024-02-08T22:38:30.134745573Z",
    "target":"tracing_actix_web::root_span_builder",
    "line":41,
    "file":"/usr/local/cargo/registry/src/index.crates.io-6f17d22bba15001f/tracing-actix-web-0.7.9/src/root_span_builder.rs",
    "http.target":"/healthz",
    "otel.name":"HTTP GET /healthz",
    "http.user_agent":"kube-probe/1.25",
    "http.method":"GET",
    "request_id":"edadd9a5-350a-47fa-a6d3-03c6610028aa",
    "http.client_ip":"10.129.0.1",
    "otel.kind":"server",
    "http.host":"10.129.0.186:8000",
    "http.route":"/healthz",
    "http.flavor":"1.1",
    "http.scheme":"http"
}

{"v":0,"name":"healthcat","msg":"[HTTP REQUEST - END]","level":30,"hostname":"healthcat-rs-57bc7f7565-gxsf2","pid":1,"time":"2024-02-08T22:38:30.134825139Z","target":"tracing_actix_web::root_span_builder","line":41,"file":"/usr/local/cargo/registry/src/index.crates.io-6f17d22bba15001f/tracing-actix-web-0.7.9/src/root_span_builder.rs","http.target":"/healthz","otel.name":"HTTP GET /healthz","otel.status_code":"OK","http.status_code":200,"elapsed_milliseconds":0,"http.user_agent":"kube-probe/1.25","http.method":"GET","request_id":"edadd9a5-350a-47fa-a6d3-03c6610028aa","http.client_ip":"10.129.0.1","otel.kind":"server","http.host":"10.129.0.186:8000","http.route":"/healthz","http.flavor":"1.1","http.scheme":"http"}
```