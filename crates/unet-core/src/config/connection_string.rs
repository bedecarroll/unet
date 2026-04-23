//! Connection-string sanitization helpers for safe diagnostics.

/// Summarize a database target without exposing raw credentials or endpoints.
#[must_use]
pub fn describe_database_target(connection_string: &str) -> String {
    let trimmed = connection_string.trim();

    if trimmed.is_empty() {
        return "database connection (value redacted)".to_string();
    }

    if let Some(sqlite_target) = describe_sqlite_target(trimmed) {
        return sqlite_target;
    }

    if let Some((scheme, remainder)) = trimmed.split_once("://") {
        return describe_network_target(scheme, remainder);
    }

    if let Some((scheme, _)) = trimmed.split_once(':') {
        return format!("{scheme} connection (value redacted)");
    }

    "database connection (value redacted)".to_string()
}

/// Replace a raw connection string inside a larger message with its redacted description.
#[must_use]
pub fn redact_connection_string(message: &str, connection_string: &str) -> String {
    if connection_string.is_empty() {
        return message.to_string();
    }

    message.replace(
        connection_string,
        &describe_database_target(connection_string),
    )
}

fn describe_sqlite_target(connection_string: &str) -> Option<String> {
    let sqlite_path = connection_string.strip_prefix("sqlite:")?;
    let path = trim_query_and_fragment(sqlite_path).trim_start_matches('/');

    if path == ":memory:" {
        return Some("sqlite connection (in-memory)".to_string());
    }

    let file_name = path.rsplit('/').find(|segment| !segment.is_empty())?;
    Some(format!("sqlite connection (file={file_name})"))
}

fn describe_network_target(scheme: &str, remainder: &str) -> String {
    let (without_fragment, has_fragment) = split_marker(remainder, '#');
    let (without_query, has_query) = split_marker(without_fragment, '?');
    let (authority, path) = without_query
        .split_once('/')
        .map_or((without_query, ""), |(authority, path)| (authority, path));
    let (credentials_present, endpoint) = authority
        .rsplit_once('@')
        .map_or((false, authority), |(_, endpoint)| (true, endpoint));

    let mut details = Vec::new();
    if !endpoint.is_empty() {
        details.push("endpoint=redacted".to_string());
    }
    if let Some(port) = parse_port(endpoint) {
        details.push(format!("port={port}"));
    }
    if let Some(database_name) = path.rsplit('/').find(|segment| !segment.is_empty()) {
        details.push(format!("database={database_name}"));
    }
    if credentials_present {
        details.push("credentials=redacted".to_string());
    }
    if has_query || has_fragment {
        details.push("query=omitted".to_string());
    }

    if details.is_empty() {
        format!("{scheme} connection (value redacted)")
    } else {
        format!("{scheme} connection ({})", details.join(", "))
    }
}

fn trim_query_and_fragment(value: &str) -> &str {
    let (without_fragment, _) = split_marker(value, '#');
    let (without_query, _) = split_marker(without_fragment, '?');
    without_query
}

fn split_marker(value: &str, marker: char) -> (&str, bool) {
    value
        .split_once(marker)
        .map_or((value, false), |(head, _)| (head, true))
}

fn parse_port(endpoint: &str) -> Option<u16> {
    if endpoint.starts_with('[') {
        let (_, port) = endpoint.split_once("]:")?;
        return port.parse().ok();
    }

    let (host, port) = endpoint.rsplit_once(':')?;
    if host.contains(':') {
        return None;
    }

    port.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::{describe_database_target, redact_connection_string};

    #[test]
    fn test_describe_database_target_redacts_credentials_for_network_urls() {
        let target = describe_database_target(
            "postgresql://demo-user:demo-pass@db.example.internal:5432/unet?sslmode=require",
        );

        assert!(target.contains("postgresql connection"));
        assert!(target.contains("credentials=redacted"));
        assert!(target.contains("endpoint=redacted"));
        assert!(target.contains("port=5432"));
        assert!(target.contains("database=unet"));
        assert!(target.contains("query=omitted"));
        assert!(!target.contains("demo-user"));
        assert!(!target.contains("demo-pass"));
        assert!(!target.contains("db.example.internal"));
    }

    #[test]
    fn test_describe_database_target_summarizes_sqlite_file_urls() {
        let target = describe_database_target("sqlite:///var/lib/unet/production.db");

        assert_eq!(target, "sqlite connection (file=production.db)");
    }

    #[test]
    fn test_describe_database_target_identifies_in_memory_sqlite() {
        let target = describe_database_target("sqlite::memory:");

        assert_eq!(target, "sqlite connection (in-memory)");
    }

    #[test]
    fn test_describe_database_target_redacts_blank_values() {
        let target = describe_database_target("   ");

        assert_eq!(target, "database connection (value redacted)");
    }

    #[test]
    fn test_describe_database_target_redacts_non_hierarchical_values() {
        let target = describe_database_target("postgresql:ssl-required");

        assert_eq!(target, "postgresql connection (value redacted)");
    }

    #[test]
    fn test_describe_database_target_redacts_minimal_network_targets() {
        let target = describe_database_target("postgresql://");

        assert_eq!(target, "postgresql connection (value redacted)");
    }

    #[test]
    fn test_describe_database_target_supports_ipv6_hosts() {
        let target =
            describe_database_target("postgresql://user:pass@[2001:db8::1]:5432/unet#fragment");

        assert!(target.contains("postgresql connection"));
        assert!(target.contains("endpoint=redacted"));
        assert!(target.contains("port=5432"));
        assert!(target.contains("database=unet"));
        assert!(target.contains("credentials=redacted"));
        assert!(target.contains("query=omitted"));
        assert!(!target.contains("2001:db8::1"));
    }

    #[test]
    fn test_redact_connection_string_replaces_raw_url_inside_error_text() {
        let connection_string =
            "postgresql://demo-user:demo-pass@db.example.internal:5432/unet?sslmode=require";
        let message = format!(
            "Failed to connect to database: Connection Error: The connection string '{connection_string}' has no supporting driver."
        );

        let sanitized = redact_connection_string(&message, connection_string);

        assert!(sanitized.contains("postgresql connection"));
        assert!(!sanitized.contains(connection_string));
        assert!(!sanitized.contains("demo-user"));
        assert!(!sanitized.contains("demo-pass"));
        assert!(!sanitized.contains("db.example.internal"));
    }

    #[test]
    fn test_redact_connection_string_keeps_message_when_connection_string_empty() {
        let message = "Failed to connect to database";

        assert_eq!(redact_connection_string(message, ""), message);
    }
}
