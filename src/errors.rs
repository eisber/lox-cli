//! Rich error types with fuzzy matching, suggestions, and doc links.
//!
//! Every config command uses these error types instead of raw `bail!()` to
//! provide actionable feedback with "did you mean?" suggestions, available
//! options, and links to relevant documentation.

/// Compute Levenshtein distance between two strings (case-insensitive).
#[allow(clippy::needless_range_loop)]
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a = a.to_lowercase();
    let b = b.to_lowercase();
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let n = a.len();
    let m = b.len();
    let mut dp = vec![vec![0usize; m + 1]; n + 1];
    for (i, row) in dp.iter_mut().enumerate().take(n + 1) {
        row[0] = i;
    }
    for j in 0..=m {
        dp[0][j] = j;
    }
    for i in 1..=n {
        for j in 1..=m {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }
    dp[n][m]
}

/// Find the closest match for `query` in `candidates`.
/// Returns `(best_match, distance)` or None if candidates is empty.
pub fn fuzzy_match<'a>(query: &str, candidates: &'a [String]) -> Option<(&'a str, usize)> {
    candidates
        .iter()
        .map(|c| (c.as_str(), levenshtein(query, c)))
        .min_by_key(|(_, d)| *d)
}

/// Suggest the closest match if the distance is reasonable (< 50% of query length).
pub fn suggest(query: &str, candidates: &[String]) -> Option<String> {
    let (best, dist) = fuzzy_match(query, candidates)?;
    let threshold = (query.len() / 2).max(3);
    if dist <= threshold {
        Some(best.to_string())
    } else {
        None
    }
}

/// Format an error with suggestions and available options.
pub fn not_found_error(
    entity: &str,
    query: &str,
    candidates: &[String],
    help_cmd: &str,
) -> anyhow::Error {
    let mut msg = format!("{} '{}' not found.", entity, query);

    if let Some(suggestion) = suggest(query, candidates) {
        msg.push_str(&format!("\n  Did you mean '{}'?", suggestion));
    }

    if candidates.len() <= 20 {
        msg.push_str(&format!("\n  Available: {}", candidates.join(", ")));
    } else {
        msg.push_str(&format!(
            "\n  {} options available. Run: {}",
            candidates.len(),
            help_cmd
        ));
    }

    anyhow::anyhow!(msg)
}

/// Format an ambiguous match error.
#[allow(dead_code)]
pub fn ambiguous_error(entity: &str, query: &str, matches: &[String]) -> anyhow::Error {
    anyhow::anyhow!(
        "{} '{}' is ambiguous — matches {} items: {}\n  Use a more specific selector (uuid:X or Type:X)",
        entity,
        query,
        matches.len(),
        matches
            .iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join(", "),
    )
}

/// Format a type mismatch error for connector wiring.
#[allow(dead_code)]
pub fn type_mismatch_error(
    source: &str,
    target: &str,
    source_kind: &str,
    target_kind: &str,
) -> anyhow::Error {
    anyhow::anyhow!(
        "Cannot wire {} → {}: connector type mismatch ({} → {}).\n  \
         Analog connectors (AQ/AI) must connect to analog.\n  \
         Digital connectors (Q/I) must connect to digital.",
        source,
        target,
        source_kind,
        target_kind,
    )
}

/// Doc reference for a control type.
#[allow(dead_code)]
pub fn doc_url(type_slug: &str) -> String {
    format!("https://www.loxone.com/enen/kb/{}/", type_slug)
}

/// Map a Loxone control type to its doc slug.
#[allow(dead_code)]
pub fn type_to_doc_slug(control_type: &str) -> Option<&'static str> {
    match control_type {
        "LightController2" => Some("lighting-controller"),
        "Switch" => Some("switch"),
        "Dimmer" => Some("dimmer"),
        "Jalousie" | "CentralJalousie" => Some("automated-blinds"),
        "Gate" | "CentralGate" => Some("gate"),
        "AlarmClock" => Some("alarm-clock"),
        "PresenceDetector" => Some("presence-detector"),
        "IRoomControllerV2" => Some("intelligent-room-controller"),
        "Alarm" => Some("alarm-system"),
        "WeatherData" | "WeatherServer" => Some("weather-service"),
        "Plugin" => Some("mqtt"),
        "GenTSensor" | "GenTActor" => Some("mqtt"),
        "Sauna" => Some("sauna"),
        "Fronius" => Some("fronius"),
        "AudioZone" => Some("audioserver"),
        _ => None,
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_identical() {
        assert_eq!(levenshtein("hello", "hello"), 0);
    }

    #[test]
    fn test_levenshtein_case_insensitive() {
        assert_eq!(levenshtein("Kitchen", "kitchen"), 0);
    }

    #[test]
    fn test_levenshtein_typo() {
        assert_eq!(levenshtein("Kitchen", "Kichen"), 1);
        assert_eq!(levenshtein("Zentral", "Znetral"), 2);
    }

    #[test]
    fn test_suggest_close_match() {
        let candidates = vec![
            "Kitchen".to_string(),
            "Bedroom".to_string(),
            "Zentral".to_string(),
        ];
        assert_eq!(suggest("Kichen", &candidates), Some("Kitchen".to_string()));
        assert_eq!(suggest("Znetral", &candidates), Some("Zentral".to_string()));
    }

    #[test]
    fn test_suggest_no_match() {
        let candidates = vec!["Kitchen".to_string(), "Bedroom".to_string()];
        assert_eq!(suggest("CompletelyDifferent", &candidates), None);
    }

    #[test]
    fn test_not_found_error_with_suggestion() {
        let candidates = vec!["Kitchen".to_string(), "Bedroom".to_string()];
        let err = not_found_error("Room", "Kichen", &candidates, "lox config room list");
        let msg = err.to_string();
        assert!(msg.contains("not found"));
        assert!(msg.contains("Kitchen"));
    }

    #[test]
    fn test_type_to_doc_slug() {
        assert_eq!(
            type_to_doc_slug("LightController2"),
            Some("lighting-controller")
        );
        assert_eq!(type_to_doc_slug("Switch"), Some("switch"));
        assert_eq!(type_to_doc_slug("UnknownType"), None);
    }

    // ── Additional error edge case tests ─────────────────────────────────

    #[test]
    fn test_levenshtein_empty_strings() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("", "xyz"), 3);
    }

    #[test]
    fn test_levenshtein_unicode() {
        assert_eq!(levenshtein("Küche", "Küche"), 0);
        assert_eq!(levenshtein("Küche", "Kuche"), 1);
    }

    #[test]
    fn test_levenshtein_completely_different() {
        assert_eq!(levenshtein("abc", "xyz"), 3);
    }

    #[test]
    fn test_fuzzy_match_empty_candidates() {
        let candidates: Vec<String> = vec![];
        assert!(fuzzy_match("query", &candidates).is_none());
    }

    #[test]
    fn test_fuzzy_match_exact() {
        let candidates = vec!["Kitchen".to_string(), "Bedroom".to_string()];
        let (best, dist) = fuzzy_match("Kitchen", &candidates).unwrap();
        assert_eq!(best, "Kitchen");
        assert_eq!(dist, 0);
    }

    #[test]
    fn test_fuzzy_match_single_candidate() {
        let candidates = vec!["Hello".to_string()];
        let (best, _) = fuzzy_match("World", &candidates).unwrap();
        assert_eq!(best, "Hello");
    }

    #[test]
    fn test_suggest_exact_match() {
        let candidates = vec!["Kitchen".to_string()];
        assert_eq!(suggest("Kitchen", &candidates), Some("Kitchen".to_string()));
    }

    #[test]
    fn test_suggest_empty_candidates() {
        let candidates: Vec<String> = vec![];
        assert_eq!(suggest("query", &candidates), None);
    }

    #[test]
    fn test_not_found_error_no_suggestion() {
        let candidates = vec!["Kitchen".to_string()];
        let err = not_found_error("Room", "ZZZZZZZZZ", &candidates, "lox config room list");
        let msg = err.to_string();
        assert!(msg.contains("not found"));
        assert!(msg.contains("Available"));
    }

    #[test]
    fn test_not_found_error_many_candidates() {
        let candidates: Vec<String> = (0..25).map(|i| format!("Room{}", i)).collect();
        let err = not_found_error("Room", "ZZZZ", &candidates, "lox config room list");
        let msg = err.to_string();
        assert!(msg.contains("25 options available"));
        assert!(msg.contains("lox config room list"));
        // Should NOT list all items
        assert!(!msg.contains("Room24"));
    }

    #[test]
    fn test_ambiguous_error() {
        let matches = vec!["Kitchen Light".to_string(), "Kitchen Fan".to_string()];
        let err = ambiguous_error("Control", "Kitchen", &matches);
        let msg = err.to_string();
        assert!(msg.contains("ambiguous"));
        assert!(msg.contains("2 items"));
        assert!(msg.contains("Kitchen Light"));
        assert!(msg.contains("Kitchen Fan"));
    }

    #[test]
    fn test_ambiguous_error_many_matches() {
        let matches: Vec<String> = (0..10).map(|i| format!("Item{}", i)).collect();
        let err = ambiguous_error("Control", "Item", &matches);
        let msg = err.to_string();
        assert!(msg.contains("10 items"));
        // Only first 5 should be shown
        assert!(msg.contains("Item4"));
        assert!(!msg.contains("Item5"));
    }

    #[test]
    fn test_type_mismatch_error() {
        let err = type_mismatch_error("LightSwitch", "DimmerInput", "digital", "analog");
        let msg = err.to_string();
        assert!(msg.contains("Cannot wire"));
        assert!(msg.contains("LightSwitch"));
        assert!(msg.contains("DimmerInput"));
        assert!(msg.contains("digital"));
        assert!(msg.contains("analog"));
    }

    #[test]
    fn test_doc_url() {
        let url = doc_url("switch");
        assert_eq!(url, "https://www.loxone.com/enen/kb/switch/");
    }

    #[test]
    fn test_type_to_doc_slug_all_known() {
        assert_eq!(type_to_doc_slug("Dimmer"), Some("dimmer"));
        assert_eq!(type_to_doc_slug("Jalousie"), Some("automated-blinds"));
        assert_eq!(
            type_to_doc_slug("CentralJalousie"),
            Some("automated-blinds")
        );
        assert_eq!(type_to_doc_slug("Gate"), Some("gate"));
        assert_eq!(type_to_doc_slug("CentralGate"), Some("gate"));
        assert_eq!(type_to_doc_slug("AlarmClock"), Some("alarm-clock"));
        assert_eq!(
            type_to_doc_slug("PresenceDetector"),
            Some("presence-detector")
        );
        assert_eq!(
            type_to_doc_slug("IRoomControllerV2"),
            Some("intelligent-room-controller")
        );
        assert_eq!(type_to_doc_slug("Alarm"), Some("alarm-system"));
        assert_eq!(type_to_doc_slug("WeatherData"), Some("weather-service"));
        assert_eq!(type_to_doc_slug("WeatherServer"), Some("weather-service"));
        assert_eq!(type_to_doc_slug("Plugin"), Some("mqtt"));
        assert_eq!(type_to_doc_slug("GenTSensor"), Some("mqtt"));
        assert_eq!(type_to_doc_slug("GenTActor"), Some("mqtt"));
        assert_eq!(type_to_doc_slug("Sauna"), Some("sauna"));
        assert_eq!(type_to_doc_slug("Fronius"), Some("fronius"));
        assert_eq!(type_to_doc_slug("AudioZone"), Some("audioserver"));
    }
}

/// Structured error for JSON output (used by agent automation)
#[allow(dead_code)]
#[derive(serde::Serialize)]
pub struct StructuredError {
    pub ok: bool,
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available: Option<Vec<String>>,
}

#[allow(dead_code)]
impl StructuredError {
    pub fn new(error: &str, message: &str) -> Self {
        Self {
            ok: false,
            error: error.to_string(),
            message: message.to_string(),
            suggestion: None,
            available: None,
        }
    }

    pub fn with_suggestion(mut self, s: &str) -> Self {
        self.suggestion = Some(s.to_string());
        self
    }

    pub fn with_available(mut self, a: Vec<String>) -> Self {
        self.available = Some(a);
        self
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self)
            .unwrap_or_else(|_| format!("{{\"error\":\"{}\"}}", self.error))
    }
}
