use crate::routes::ApiResult;

use prometheus::{Encoder, HistogramVec, IntCounterVec, TextEncoder};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Data, Request, Response};
use std::time::Instant;

lazy_static! {
    static ref HTTP_REQUESTS: IntCounterVec = register_int_counter_vec!(
        "http_requests",
        "Total number of HTTP requests",
        &["endpoint", "method", "status"]
    )
    .unwrap();
    static ref HTTP_REQUESTS_DURATION: HistogramVec = register_histogram_vec!(
        "http_requests_duration",
        "HTTP request duration taken in seconds",
        &["endpoint", "method", "status"]
    )
    .unwrap();
    pub static ref ANDESITE_EVENTS: IntCounterVec = register_int_counter_vec!(
        "andesite_events",
        "Events received through the Andesite websocket",
        &["type"]
    )
    .unwrap();
    pub static ref PLAYED_TRACKS: IntCounterVec = register_int_counter_vec!(
        "played_tracks",
        "All the tracks played using the bot",
        &["title", "length"]
    )
    .unwrap();
    pub static ref VOICE_CLOSES: IntCounterVec = register_int_counter_vec!(
        "voice_closes",
        "Discord voice gateway close events",
        &["code"]
    )
    .unwrap();
}

#[derive(Debug, Clone)]
pub struct Metrics;

impl Metrics {
    pub fn fairing() -> Self {
        Self
    }
}

#[derive(Debug, Clone)]
struct RequestTimer(Option<Instant>);

impl Fairing for Metrics {
    fn info(&self) -> Info {
        Info {
            name: "Prometheus metrics collection",
            kind: Kind::Request | Kind::Response,
        }
    }

    fn on_request(&self, request: &mut Request<'_>, _: &Data) {
        request.local_cache(|| RequestTimer(Some(Instant::now())));
    }

    fn on_response(&self, request: &Request<'_>, response: &mut Response<'_>) {
        if let Some(route) = request.route() {
            let endpoint = route.uri.to_string();
            let method = request.method().as_str();
            let status = response.status().code.to_string();

            if endpoint == "/metrics" {
                return;
            }

            HTTP_REQUESTS
                .with_label_values(&[endpoint.as_str(), method, status.as_str()])
                .inc();

            let start_time = request.local_cache(|| RequestTimer(None));

            if let Some(duration) = start_time.0.map(|st| st.elapsed()) {
                HTTP_REQUESTS_DURATION
                    .with_label_values(&[endpoint.as_str(), method, status.as_str()])
                    .observe(duration.as_secs_f64());
            }
        }
    }
}

#[get("/metrics")]
pub fn get_metrics() -> ApiResult<String> {
    let mut buffer = vec![];
    let metrics = prometheus::gather();

    TextEncoder::new().encode(metrics.as_slice(), &mut buffer)?;

    Ok(String::from_utf8(buffer)?)
}
