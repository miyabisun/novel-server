use chrono::{DateTime, FixedOffset, NaiveDateTime, SecondsFormat, TimeZone, Utc};

const JST_SECONDS: i32 = 9 * 60 * 60;

/// Parse an upstream publication time into an absolute Unix timestamp.
///
/// Zoned RFC 3339 values keep their declared offset. Naive values from the
/// supported Japanese novel services are interpreted as JST.
pub(crate) fn parse_upstream_timestamp(value: &str) -> Option<i64> {
    let value = value.trim();
    if let Ok(timestamp) = DateTime::parse_from_rfc3339(value) {
        return Some(timestamp.timestamp());
    }

    let naive = ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%d %H:%M:%S%.f"]
        .into_iter()
        .find_map(|format| NaiveDateTime::parse_from_str(value, format).ok())?;
    FixedOffset::east_opt(JST_SECONDS)?
        .from_local_datetime(&naive)
        .single()
        .map(|timestamp| timestamp.timestamp())
}

pub(crate) fn unix_timestamp_to_rfc3339(timestamp: i64) -> Option<String> {
    DateTime::<Utc>::from_timestamp(timestamp, 0)
        .map(|value| value.to_rfc3339_opts(SecondsFormat::Secs, true))
}

pub(crate) fn unix_timestamp_to_rfc2822(timestamp: i64) -> Option<String> {
    DateTime::<Utc>::from_timestamp(timestamp, 0).map(|value| value.to_rfc2822())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoned_input_preserves_the_instant() {
        assert_eq!(
            parse_upstream_timestamp("2026-03-14T09:00:00+09:00"),
            parse_upstream_timestamp("2026-03-14T00:00:00Z")
        );
    }

    #[test]
    fn naive_japanese_input_is_interpreted_as_jst() {
        let timestamp = parse_upstream_timestamp("2026-03-14 09:00:00").unwrap();
        assert_eq!(
            unix_timestamp_to_rfc3339(timestamp).as_deref(),
            Some("2026-03-14T00:00:00Z")
        );
    }

    #[test]
    fn malformed_input_is_rejected() {
        assert_eq!(parse_upstream_timestamp("not a date"), None);
        assert_eq!(parse_upstream_timestamp("2026-03-14"), None);
    }
}
