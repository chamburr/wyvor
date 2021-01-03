use crate::routes::ApiResult;

use actix_web::body::{BodySize, MessageBody, ResponseBody};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::get;
use actix_web::http::{Method, StatusCode};
use actix_web::web::Bytes;
use futures::future::{ok, Ready};
use futures::Future;
use lazy_static::lazy_static;
use pin_project::{pin_project, pinned_drop};
use prometheus::{
    register_histogram_vec, register_int_counter_vec, Encoder, HistogramVec, IntCounterVec,
    TextEncoder,
};
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
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
}

pub struct Metrics;

impl<S, B> Transform<S> for Metrics
where
    B: MessageBody,
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<StreamLog<B>>;
    type Error = actix_web::Error;
    type Transform = MetricsMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(MetricsMiddleware { service })
    }
}

pub struct MetricsMiddleware<S> {
    service: S,
}

impl<S, B> Service for MetricsMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    B: MessageBody,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<StreamLog<B>>;
    type Error = S::Error;
    type Future = LoggerResponse<S, B>;

    fn poll_ready(&mut self, ct: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ct)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        LoggerResponse {
            fut: self.service.call(req),
            clock: Instant::now(),
            _t: PhantomData,
        }
    }
}

#[pin_project]
pub struct LoggerResponse<S, B>
where
    B: MessageBody,
    S: Service,
{
    #[pin]
    fut: S::Future,
    clock: Instant,
    _t: PhantomData<(B,)>,
}

impl<S, B> Future for LoggerResponse<S, B>
where
    B: MessageBody,
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
{
    type Output = Result<ServiceResponse<StreamLog<B>>, actix_web::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let res = match futures::ready!(this.fut.poll(cx)) {
            Ok(res) => res,
            Err(e) => return Poll::Ready(Err(e)),
        };

        let clock = *this.clock;
        let req = res.request();
        let method = req.method().clone();
        let endpoint = req.match_pattern();

        Poll::Ready(Ok(res.map_body(move |head, body| {
            ResponseBody::Body(StreamLog {
                body,
                size: 0,
                clock,
                status: head.status,
                endpoint,
                method,
            })
        })))
    }
}

#[pin_project(PinnedDrop)]
pub struct StreamLog<B> {
    #[pin]
    body: ResponseBody<B>,
    size: usize,
    clock: Instant,
    status: StatusCode,
    endpoint: Option<String>,
    method: Method,
}

#[pinned_drop]
impl<B> PinnedDrop for StreamLog<B> {
    fn drop(self: Pin<&mut Self>) {
        if let Some(endpoint) = &self.endpoint {
            if endpoint == "/metrics" {
                return;
            }

            HTTP_REQUESTS
                .with_label_values(&[
                    endpoint.as_str(),
                    self.method.as_str(),
                    self.status.as_str(),
                ])
                .inc();

            HTTP_REQUESTS_DURATION
                .with_label_values(&[
                    endpoint.as_str(),
                    self.method.as_str(),
                    self.status.as_str(),
                ])
                .observe(self.clock.elapsed().as_secs_f64());
        }
    }
}

impl<B: MessageBody> MessageBody for StreamLog<B> {
    fn size(&self) -> BodySize {
        self.body.size()
    }

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, actix_web::Error>>> {
        let this = self.project();
        match MessageBody::poll_next(this.body, cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                *this.size += chunk.len();
                Poll::Ready(Some(Ok(chunk)))
            },
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(err))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[get("/metrics")]
pub async fn get_metrics() -> ApiResult<String> {
    let mut buffer = vec![];
    let metrics = prometheus::gather();

    TextEncoder::new().encode(metrics.as_slice(), &mut buffer)?;

    Ok(String::from_utf8(buffer)?)
}
