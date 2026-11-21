#![cfg(feature = "runtime")]

use lox_core::runtime::session_endpoints::{
    EndpointError, EndpointIssue, EndpointKind, ProxyPolicy, SessionEndpoints,
};

const HTTP: &str = "https://http.example.test";
const WS: &str = "wss://ws.example.test/ws/rfc6455";

#[test]
fn canonicalizes_explicit_routes_without_inferring_the_other_route() {
    let endpoints = SessionEndpoints::parse(
        "HTTP://192.0.2.10:80/site/~status/",
        "wss://[2001:0db8:0:0::2]:8443/proxy/ws",
        ProxyPolicy::Direct,
    )
    .unwrap();
    assert_eq!(endpoints.http_url(), "http://192.0.2.10/site/~status/");
    assert_eq!(
        endpoints.websocket_url(),
        "wss://[2001:db8::2]:8443/proxy/ws"
    );
    assert_eq!(endpoints.proxy_policy(), ProxyPolicy::Direct);

    let endpoints = SessionEndpoints::parse(
        "https://[::1]:65535/prefix",
        "ws://LAN.Example.test:80",
        ProxyPolicy::System,
    )
    .unwrap();
    assert_eq!(endpoints.http_url(), "https://[::1]:65535/prefix");
    assert_eq!(endpoints.websocket_url(), "ws://lan.example.test/");
    assert_eq!(endpoints.proxy_policy(), ProxyPolicy::System);
}

#[test]
fn requires_an_explicit_scheme_for_each_protocol() {
    for (http, ws, endpoint) in [
        ("host.example.test:443", WS, EndpointKind::Http),
        ("https:host.example.test", WS, EndpointKind::Http),
        (WS, WS, EndpointKind::Http),
        (HTTP, HTTP, EndpointKind::WebSocket),
        (HTTP, "//host.example.test", EndpointKind::WebSocket),
        (HTTP, "ftp://host.example.test", EndpointKind::WebSocket),
    ] {
        assert_eq!(
            SessionEndpoints::parse(http, ws, ProxyPolicy::Direct).unwrap_err(),
            EndpointError {
                endpoint,
                issue: EndpointIssue::Scheme,
            }
        );
    }
}

#[test]
fn rejects_malformed_or_ambiguous_authorities_and_paths() {
    for (input, issue) in [
        ("https://", EndpointIssue::Authority),
        ("https:///host.example.test", EndpointIssue::Authority),
        ("https://host.example.test:", EndpointIssue::Authority),
        ("https://host.example.test:0", EndpointIssue::Authority),
        ("https://host.example.test:65536", EndpointIssue::Authority),
        ("https://host.example.test:+80", EndpointIssue::Authority),
        ("https://host..example.test", EndpointIssue::Authority),
        ("https://ab\tcd.example.test", EndpointIssue::Authority),
        ("https://host.example.test\\route", EndpointIssue::Authority),
        ("https://192.0.2", EndpointIssue::Authority),
        ("https://0300.0.2.10", EndpointIssue::Authority),
        ("https://2001:db8::1", EndpointIssue::Authority),
        ("https://[2001:db8::1", EndpointIssue::Authority),
        ("https://[fe80::1%25zone]", EndpointIssue::Authority),
        ("https://host.example.test/a//b", EndpointIssue::Path),
        ("https://host.example.test/a/../b", EndpointIssue::Path),
        ("https://host.example.test/a/./b", EndpointIssue::Path),
        ("https://host.example.test/a%2fb", EndpointIssue::Path),
        ("https://host.example.test/a%2", EndpointIssue::Path),
        ("https://host.example.test/a\\b", EndpointIssue::Path),
        ("https://host.example.test/a=b", EndpointIssue::Path),
    ] {
        assert_eq!(
            SessionEndpoints::parse(input, WS, ProxyPolicy::Direct).unwrap_err(),
            EndpointError {
                endpoint: EndpointKind::Http,
                issue,
            },
            "{input:?}"
        );
    }
}

#[test]
fn refuses_url_credentials_and_omits_route_data_from_diagnostics() {
    for input in [
        "wss://private-user:private-password@private-host.test/route",
        "wss://private-user@private-host.test",
        "wss://@private-host.test",
        "wss://private-host.test/route?custom=private-value",
        "wss://private-host.test/route#private-value",
        "wss://private-host.test/route?",
        "wss://private-host.test/route#",
    ] {
        let error = SessionEndpoints::parse(HTTP, input, ProxyPolicy::Direct).unwrap_err();
        assert_eq!(
            error,
            EndpointError {
                endpoint: EndpointKind::WebSocket,
                issue: EndpointIssue::UserInfoQueryOrFragment,
            }
        );
        assert!(!format!("{error} {error:?}").contains("private-"));
        assert!(std::error::Error::source(&error).is_none());
    }

    let endpoints = SessionEndpoints::parse(
        "https://private-host.test/private-prefix",
        WS,
        ProxyPolicy::Direct,
    )
    .unwrap();
    let debug = format!("{endpoints:?}");
    assert!(!debug.contains("private-"));
    assert!(!debug.contains(WS));
}
