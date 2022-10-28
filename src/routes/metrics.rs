use log::error;
use prometheus::{self, Encoder, TextEncoder};
use rocket::http::Status;

#[get("/metrics")]
pub async fn metrics() -> Result<String, Status> {
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    encoder
        .encode(&metric_families, &mut buffer)
        .map_err(|e| {
            error!("{:?}", e);
            Status::InternalServerError
        })
        .and(String::from_utf8(buffer.clone()).map_err(|e| {
            error!("{:?}", e);
            Status::InternalServerError
        }))
        .map_or_else(
            |e| {
                error!("{:?}", e);
                Err(Status::InternalServerError)
            },
            |data| Ok(data),
        )
}
