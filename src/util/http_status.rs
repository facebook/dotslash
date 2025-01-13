/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

//! 4xx and 5xx HTTP status codes as an Error.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum HttpStatus {
    #[error("400 Bad Request")]
    BadRequest,

    #[error("401 Unauthorized")]
    Unauthorized,

    #[error("402 Payment Required")]
    PaymentRequired,

    #[error("403 Forbidden")]
    Forbidden,

    #[error("404 Not Found")]
    NotFound,

    #[error("405 Method Not Allowed")]
    MethodNotAllowed,

    #[error("406 Not Acceptable")]
    NotAcceptable,

    #[error("407 Proxy Authentication Required")]
    ProxyAuthenticationRequired,

    #[error("408 Request Timeout")]
    RequestTimeout,

    #[error("409 Conflict")]
    Conflict,

    #[error("410 Gone")]
    Gone,

    #[error("411 Length Required")]
    LengthRequired,

    #[error("412 Precondition Failed")]
    PreconditionFailed,

    #[error("413 Request Entity Too Large")]
    RequestEntityTooLarge,

    #[error("414 Request-URI Too Long")]
    RequestUriTooLong,

    #[error("415 Unsupported Media Type")]
    UnsupportedMediaType,

    #[error("416 Requested Range Not Satisfiable")]
    RequestedRangeNotSatisfiable,

    #[error("417 Expectation Failed")]
    ExpectationFailed,

    #[error("418 I'm a teapot (RFC 2324)")]
    ImaTeapotRfc2324,

    #[error("420 Enhance Your Calm (Twitter)")]
    EnhanceYourCalmTwitter,

    #[error("422 Unprocessable Entity (WebDAV)")]
    UnprocessableEntityWebDav,

    #[error("423 Locked (WebDAV)")]
    LockedWebDav,

    #[error("424 Failed Dependency (WebDAV)")]
    FailedDependencyWebDav,

    #[error("425 Reserved for WebDAV")]
    ReservedForWebDav,

    #[error("426 Upgrade Required")]
    UpgradeRequired,

    #[error("428 Precondition Required")]
    PreconditionRequired,

    #[error("429 Too Many Requests")]
    TooManyRequests,

    #[error("431 Request Header Fields Too Large")]
    RequestHeaderFieldsTooLarge,

    #[error("444 No Response (Nginx)")]
    NoResponseNginx,

    #[error("449 Retry With (Microsoft)")]
    RetryWithMicrosoft,

    #[error("450 Blocked by Windows Parental Controls (Microsoft)")]
    BlockedByWindowsParentalControlsMicrosoft,

    #[error("451 Unavailable For Legal Reasons")]
    UnavailableForLegalReasons,

    #[error("499 Client Closed Request (Nginx)")]
    ClientClosedRequestNginx,

    #[error("500 Internal Server Error")]
    InternalServerError,

    #[error("501 Not Implemented")]
    NotImplemented,

    #[error("502 Bad Gateway")]
    BadGateway,

    #[error("503 Service Unavailable")]
    ServiceUnavailable,

    #[error("504 Gateway Timeout")]
    GatewayTimeout,

    #[error("505 HTTP Version Not Supported")]
    HttpVersionNotSupported,

    #[error("506 Variant Also Negotiates (Experimental)")]
    VariantAlsoNegotiatesExperimental,

    #[error("507 Insufficient Storage (WebDAV)")]
    InsufficientStorageWebDav,

    #[error("508 Loop Detected (WebDAV)")]
    LoopDetectedWebDav,

    #[error("509 Bandwidth Limit Exceeded (Apache)")]
    BandwidthLimitExceededApache,

    #[error("510 Not Extended")]
    NotExtended,

    #[error("511 Network Authentication Required")]
    NetworkAuthenticationRequired,

    #[error("598 Network read timeout error")]
    NetworkReadTimeoutError,

    #[error("599 Network connect timeout error")]
    NetworkConnectTimeoutError,

    #[error("unknown HTTP status {0}")]
    Unknown(usize),
}

impl From<usize> for HttpStatus {
    fn from(http_status: usize) -> Self {
        match http_status {
            400 => Self::BadRequest,
            401 => Self::Unauthorized,
            402 => Self::PaymentRequired,
            403 => Self::Forbidden,
            404 => Self::NotFound,
            405 => Self::MethodNotAllowed,
            406 => Self::NotAcceptable,
            407 => Self::ProxyAuthenticationRequired,
            408 => Self::RequestTimeout,
            409 => Self::Conflict,
            410 => Self::Gone,
            411 => Self::LengthRequired,
            412 => Self::PreconditionFailed,
            413 => Self::RequestEntityTooLarge,
            414 => Self::RequestUriTooLong,
            415 => Self::UnsupportedMediaType,
            416 => Self::RequestedRangeNotSatisfiable,
            417 => Self::ExpectationFailed,
            418 => Self::ImaTeapotRfc2324,
            420 => Self::EnhanceYourCalmTwitter,
            422 => Self::UnprocessableEntityWebDav,
            423 => Self::LockedWebDav,
            424 => Self::FailedDependencyWebDav,
            425 => Self::ReservedForWebDav,
            426 => Self::UpgradeRequired,
            428 => Self::PreconditionRequired,
            429 => Self::TooManyRequests,
            431 => Self::RequestHeaderFieldsTooLarge,
            444 => Self::NoResponseNginx,
            449 => Self::RetryWithMicrosoft,
            450 => Self::BlockedByWindowsParentalControlsMicrosoft,
            451 => Self::UnavailableForLegalReasons,
            499 => Self::ClientClosedRequestNginx,
            500 => Self::InternalServerError,
            501 => Self::NotImplemented,
            502 => Self::BadGateway,
            503 => Self::ServiceUnavailable,
            504 => Self::GatewayTimeout,
            505 => Self::HttpVersionNotSupported,
            506 => Self::VariantAlsoNegotiatesExperimental,
            507 => Self::InsufficientStorageWebDav,
            508 => Self::LoopDetectedWebDav,
            509 => Self::BandwidthLimitExceededApache,
            510 => Self::NotExtended,
            511 => Self::NetworkAuthenticationRequired,
            598 => Self::NetworkReadTimeoutError,
            599 => Self::NetworkConnectTimeoutError,
            _ => Self::Unknown(http_status),
        }
    }
}
