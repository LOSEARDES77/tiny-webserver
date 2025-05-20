use std::io::Cursor;

use crate::fmt::{fmt_duration, fmt_method, fmt_statuscode};

pub fn log_request(rq: &tiny_http::Request) {
    println!(
        "{} {} {}",
        fmt_method(rq.method()),
        rq.url(),
        if let Some(addr) = rq.remote_addr() {
            addr.to_string()
        } else {
            "unknown".to_string()
        }
    );
}

pub fn log_response(
    rq: &tiny_http::Request,
    rp: &tiny_http::Response<Cursor<Vec<u8>>>,
    duration: std::time::Duration,
) {
    println!(
        "{} {} {} {} {} {}",
        fmt_method(rq.method()),
        rq.url(),
        if let Some(addr) = rq.remote_addr() {
            addr.to_string()
        } else {
            "unknown".to_string()
        },
        fmt_statuscode(rp.status_code().0),
        rp.status_code().default_reason_phrase(),
        fmt_duration(duration),
    );
}
