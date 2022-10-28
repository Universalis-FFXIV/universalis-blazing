use anyhow::{Context, Result};
use prometheus::{register_int_counter_vec, IntCounterVec};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Method;
use rocket::{Request, Response};

pub struct Metrics {
    requests_received: IntCounterVec,
}

impl Metrics {
    pub fn new() -> Result<Self> {
        let requests_received = register_int_counter_vec!(
            "universalis_blazing_reqs_received",
            "The total number of requests received by the BLAZING FAST server.",
            &["code", "method", "path"],
        )
        .context("Failed to create request counter")?;
        Ok(Self { requests_received })
    }
}

fn method_to_str(method: Method) -> String {
    match method {
        Method::Get => "Get".to_string(),
        Method::Post => "Post".to_string(),
        Method::Put => "Put".to_string(),
        Method::Delete => "Delete".to_string(),
        Method::Options => "Options".to_string(),
        Method::Head => "Head".to_string(),
        Method::Trace => "Trace".to_string(),
        Method::Connect => "Connect".to_string(),
        Method::Patch => "Patch".to_string(),
    }
}

#[rocket::async_trait]
impl Fairing for Metrics {
    fn info(&self) -> Info {
        Info {
            name: "Request Counter",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        // Don't log requests to nonexistent paths
        if request.route().is_none() {
            return;
        }

        let code = response.status().code.to_string();
        let method = method_to_str(request.method());
        let path = request
            .route()
            .map_or_else(|| String::new(), |r| r.uri.path().to_string());
        let labeled_counter = self
            .requests_received
            .with_label_values(&[&code, &method, &path]);
        labeled_counter.inc();
    }
}
