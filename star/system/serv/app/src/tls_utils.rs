use openssl::ssl::SslAlert;

use crate::filter::StarFilterErrorStatus;

pub fn alert_for_filter_error(status: &StarFilterErrorStatus) -> SslAlert {
    match status {
        StarFilterErrorStatus::InvalidToken => SslAlert::DECODE_ERROR,
        StarFilterErrorStatus::InvalidRoute => SslAlert::ILLEGAL_PARAMETER,
        StarFilterErrorStatus::DuplicatedToken => SslAlert::ILLEGAL_PARAMETER,
        StarFilterErrorStatus::Internal => SslAlert::DECODE_ERROR,
    }
}

pub fn filter_error_name(status: &StarFilterErrorStatus) -> &'static str {
    match status {
        StarFilterErrorStatus::InvalidToken => "InvalidToken",
        StarFilterErrorStatus::InvalidRoute => "InvalidRoute",
        StarFilterErrorStatus::DuplicatedToken => "DuplicatedToken",
        StarFilterErrorStatus::Internal => "Internal",
    }
}

pub fn alert_name(alert: SslAlert) -> &'static str {
    match alert {
        SslAlert::DECODE_ERROR => "DECODE_ERROR",
        SslAlert::ILLEGAL_PARAMETER => "ILLEGAL_PARAMETER",
        _ => "OTHER",
    }
}
