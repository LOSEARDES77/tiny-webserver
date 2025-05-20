use colored::{ColoredString, Colorize};

pub fn fmt_statuscode(code: u16) -> ColoredString {
    match code {
        200..=299 => format!("{}", code).green(),
        300..=399 => format!("{}", code).yellow(),
        400..=499 => format!("{}", code).red(),
        500..=599 => format!("{}", code).bright_red(),
        _ => format!("{}", code).normal(),
    }
}

pub fn fmt_method(method: &tiny_http::Method) -> ColoredString {
    match method {
        tiny_http::Method::Get => "GET".green(),
        tiny_http::Method::Post => "POST".yellow(),
        tiny_http::Method::Put => "PUT".red(),
        tiny_http::Method::Delete => "DELETE".bright_red(),
        tiny_http::Method::Head => "HEAD".normal(),
        tiny_http::Method::Options => "OPTIONS".normal(),
        _ => format!("{}", method).normal(),
    }
}

pub fn fmt_duration(duration: std::time::Duration) -> String {
    let nanos = duration.as_nanos();
    let micros = duration.as_micros();
    let millis = duration.as_millis();
    let seconds = duration.as_secs();

    if seconds > 1 {
        format!("{}.{:02}s", seconds, millis)
    } else if millis > 1 {
        format!("{}.{:02}ms", millis, micros % 1000)
    } else if micros > 1 {
        format!("{}.{:02}μs", micros, nanos % 1000)
    } else {
        format!("{}ns", nanos)
    }
}
