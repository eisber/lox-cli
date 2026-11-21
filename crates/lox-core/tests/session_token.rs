#![cfg(feature = "runtime")]

use lox_core::runtime::session_endpoints::{ProxyPolicy, SessionEndpoints};
use lox_core::runtime::session_token::{
    TokenExpiry, TokenLookup, TokenSlot, TokenSlotError, TokenSubject,
};
use std::time::{Duration, UNIX_EPOCH};

const HTTP: &str = "https://http.example.test/api";
const WS: &str = "wss://[2001:db8::2]/ws";
const PRINCIPAL: &str = " Élodie ";
const TOKEN: &str = " opaque synthetic token\n";

#[test]
fn matches_only_canonical_endpoints_policy_and_exact_principal() {
    let routes = SessionEndpoints::parse(HTTP, WS, ProxyPolicy::Direct).unwrap();
    let subject = TokenSubject::new(&routes, PRINCIPAL).unwrap();
    let mut slot = TokenSlot::new(&subject);
    let now = UNIX_EPOCH;
    let expiry = TokenExpiry::At(now + Duration::from_secs(1));
    slot.replace(&subject, TOKEN.to_owned(), expiry).unwrap();

    let equivalent = SessionEndpoints::parse(
        "HTTPS://HTTP.Example.TEST:443/api",
        "WSS://[2001:0db8:0:0::2]:443/ws",
        ProxyPolicy::Direct,
    )
    .unwrap();
    let same = TokenSubject::new(&equivalent, PRINCIPAL).unwrap();
    let TokenLookup::Match(access) = slot.lookup(&same, now) else {
        panic!("canonical routes did not match");
    };
    assert_eq!(access.expose_token(), TOKEN);

    for (http, ws) in [
        ("http://http.example.test:443/api", WS),
        ("https://other.example.test/api", WS),
        ("https://http.example.test:444/api", WS),
        ("https://http.example.test/Api", WS),
        ("https://http.example.test/api/", WS),
        (HTTP, "ws://[2001:db8::2]:443/ws"),
        (HTTP, "wss://[2001:db8::3]/ws"),
        (HTTP, "wss://[2001:db8::2]:444/ws"),
        (HTTP, "wss://[2001:db8::2]/other"),
    ] {
        let changed = SessionEndpoints::parse(http, ws, ProxyPolicy::Direct).unwrap();
        let other = TokenSubject::new(&changed, PRINCIPAL).unwrap();
        assert!(matches!(
            slot.lookup(&other, now),
            TokenLookup::SubjectMismatch
        ));
    }

    for principal in ["Élodie", " élodie ", " E\u{301}lodie ", " different "] {
        let other = TokenSubject::new(&routes, principal).unwrap();
        assert!(matches!(
            slot.lookup(&other, now),
            TokenLookup::SubjectMismatch
        ));
    }
    let system = SessionEndpoints::parse(HTTP, WS, ProxyPolicy::System).unwrap();
    let other = TokenSubject::new(&system, PRINCIPAL).unwrap();
    assert!(matches!(
        slot.lookup(&other, now),
        TokenLookup::SubjectMismatch
    ));
}

#[test]
fn replacement_and_clear_preserve_scope_and_rejected_mutations_keep_the_token() {
    let routes = SessionEndpoints::parse(HTTP, WS, ProxyPolicy::Direct).unwrap();
    assert_eq!(
        TokenSubject::new(&routes, "").unwrap_err(),
        TokenSlotError::EmptyPrincipal
    );
    let subject = TokenSubject::new(&routes, PRINCIPAL).unwrap();
    let other = TokenSubject::new(&routes, " ").unwrap();
    let mut slot = TokenSlot::new(&subject);
    let now = UNIX_EPOCH;
    let expiry = TokenExpiry::At(now + Duration::from_secs(1));
    assert!(matches!(slot.lookup(&subject, now), TokenLookup::Empty));
    slot.replace(&subject, TOKEN.to_owned(), expiry).unwrap();

    assert_eq!(
        slot.replace(&other, "different synthetic token".into(), expiry),
        Err(TokenSlotError::SubjectMismatch)
    );
    assert_eq!(slot.clear(&other), Err(TokenSlotError::SubjectMismatch));
    assert_eq!(
        slot.replace(&subject, String::new(), expiry),
        Err(TokenSlotError::EmptyToken)
    );
    let TokenLookup::Match(access) = slot.lookup(&subject, now) else {
        panic!("rejected mutation changed the slot");
    };
    assert_eq!(access.expose_token(), TOKEN);

    slot.replace(&subject, "replacement".into(), expiry)
        .unwrap();
    let TokenLookup::Match(access) = slot.lookup(&subject, now) else {
        panic!("replacement was not available");
    };
    assert_eq!(access.expose_token(), "replacement");

    slot.clear(&subject).unwrap();
    assert!(matches!(slot.lookup(&subject, now), TokenLookup::Empty));
    assert_eq!(
        slot.replace(&other, "other".into(), expiry),
        Err(TokenSlotError::SubjectMismatch)
    );
    assert!(matches!(slot.lookup(&subject, now), TokenLookup::Empty));
}

#[test]
fn checked_epoch_and_expiry_are_fail_closed_at_the_boundary() {
    const LOXONE_EPOCH: u64 = 1_230_768_000;
    let routes = SessionEndpoints::parse(HTTP, WS, ProxyPolicy::Direct).unwrap();
    let subject = TokenSubject::new(&routes, PRINCIPAL).unwrap();
    let mut slot = TokenSlot::new(&subject);
    let expires_at = UNIX_EPOCH + Duration::from_secs(LOXONE_EPOCH + 7);
    assert_eq!(
        TokenExpiry::from_loxone_seconds(0).unwrap(),
        TokenExpiry::At(UNIX_EPOCH + Duration::from_secs(LOXONE_EPOCH))
    );
    assert_eq!(
        TokenExpiry::from_loxone_seconds(u64::MAX).unwrap_err(),
        TokenSlotError::ExpiryOutOfRange
    );
    assert_eq!(
        TokenExpiry::from_loxone_seconds(u64::MAX - LOXONE_EPOCH).unwrap_err(),
        TokenSlotError::ExpiryOutOfRange
    );
    let expiry = TokenExpiry::from_loxone_seconds(7).unwrap();
    slot.replace(&subject, TOKEN.to_owned(), expiry).unwrap();
    let tick = Duration::from_millis(1);
    assert!(matches!(
        slot.lookup(&subject, expires_at - tick),
        TokenLookup::Match(_)
    ));
    for now in [expires_at, expires_at + tick] {
        assert!(matches!(slot.lookup(&subject, now), TokenLookup::Expired));
    }
    slot.replace(&subject, "unknown lifetime".into(), TokenExpiry::Unknown)
        .unwrap();
    assert!(matches!(
        slot.lookup(&subject, expires_at),
        TokenLookup::UnknownExpiry
    ));
    let other = TokenSubject::new(&routes, "another account").unwrap();
    assert!(matches!(
        slot.lookup(&other, expires_at),
        TokenLookup::SubjectMismatch
    ));
}

#[test]
fn diagnostics_do_not_render_token_or_subject_data() {
    let routes = SessionEndpoints::parse(HTTP, WS, ProxyPolicy::Direct).unwrap();
    let subject = TokenSubject::new(&routes, PRINCIPAL).unwrap();
    let other = TokenSubject::new(&routes, " other synthetic account ").unwrap();
    let mut slot = TokenSlot::new(&subject);
    let now = UNIX_EPOCH;
    let expiry = TokenExpiry::At(now + Duration::from_secs(1));
    slot.replace(&subject, TOKEN.to_owned(), expiry).unwrap();
    let error = slot.clear(&other).unwrap_err();
    let lookup = slot.lookup(&subject, now);
    let TokenLookup::Match(ref access) = lookup else {
        panic!("expected successful lookup");
    };
    let diagnostics = format!("{subject:#?} {slot:#?} {lookup:?} {access:#?} {error} {error:?}");
    for private in [TOKEN.trim(), PRINCIPAL.trim(), "http.example.test", "/ws"] {
        assert!(!diagnostics.contains(private));
    }
    assert!(std::error::Error::source(&error).is_none());
}
