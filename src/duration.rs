use std::time::{Duration, SystemTime};
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedDuration {
    Permanent,
    Timed(SystemTime),
}
/// # Rules
///  Empty string or `"permanent"` -> `Ok(Permanent)`
///  `"<number><suffix>"` where suffix is one of {s, m, h, d} -> `Ok(Timed(now + duration))`
///  Zero length durations (e.g. `0s`) should be rejected with an error.
///  Anything else -> `Err` with a descriptive message shown in the UI.
pub fn parse_duration(input: &str) -> Result<ParsedDuration, String> {
    let trimmed = input.trim().to_lowercase();
    if trimmed.is_empty() || trimmed == "permanent" {
        return Ok(ParsedDuration::Permanent);
    }
    if trimmed.len() < 2 {
        return Err(format!(
            "'{}' is not a valid duration. Use a number followed by s, m, h, or d (e.g. 30m).",
            trimmed
        ));
    }
    let (numeric_part, multiplier_secs): (&str, u64) = match trimmed.chars().last() {
        Some('s') => (&trimmed[..trimmed.len() - 1], 1),
        Some('m') => (&trimmed[..trimmed.len() - 1], 60),
        Some('h') => (&trimmed[..trimmed.len() - 1], 3_600),
        Some('d') => (&trimmed[..trimmed.len() - 1], 86_400),
        Some(c) => {
            return Err(format!(
                "'{}' is not a recognised duration suffix. Use s (seconds), m (minutes), h (hours), or d (days).",
                c
            ))
        }
        None => unreachable!("already checked len >= 2"),
    };
    let value: u64 = numeric_part.parse().map_err(|_| {
        format!(
            "'{}' is not a valid number. Duration must be a positive integer followed by s, m, h, or d.",
            numeric_part
        )
    })?;
    if value == 0 {
        return Err(
            "Block duration must be greater than zero. Use a positive number (e.g. 1m, 2h)."
                .to_string(),
        );
    }

    let total_secs = value
        .checked_mul(multiplier_secs)
        .ok_or_else(|| "Duration is too large to represent.".to_string())?;

    let duration = Duration::from_secs(total_secs);
    let expiry = SystemTime::now()
        .checked_add(duration)
        .ok_or_else(|| "Duration overflows the system clock.".to_string())?;

    Ok(ParsedDuration::Timed(expiry))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_permanent() {
        assert_eq!(parse_duration("").unwrap(), ParsedDuration::Permanent);
    }

    #[test]
    fn keyword_permanent() {
        assert_eq!(
            parse_duration("permanent").unwrap(),
            ParsedDuration::Permanent
        );
        assert_eq!(
            parse_duration("  PERMANENT  ").unwrap(),
            ParsedDuration::Permanent
        );
    }

    #[test]
    fn valid_suffixes_produce_timed() {
        assert!(matches!(
            parse_duration("30s").unwrap(),
            ParsedDuration::Timed(_)
        ));
        assert!(matches!(
            parse_duration("5m").unwrap(),
            ParsedDuration::Timed(_)
        ));
        assert!(matches!(
            parse_duration("2h").unwrap(),
            ParsedDuration::Timed(_)
        ));
        assert!(matches!(
            parse_duration("1d").unwrap(),
            ParsedDuration::Timed(_)
        ));
    }

    #[test]
    fn zero_duration_is_error() {
        assert!(parse_duration("0s").is_err());
        assert!(parse_duration("0m").is_err());
        assert!(parse_duration("0h").is_err());
        assert!(parse_duration("0d").is_err());
    }

    #[test]
    fn bad_suffix_is_error() {
        assert!(parse_duration("10x").is_err());
        assert!(parse_duration("5y").is_err());
    }

    #[test]
    fn non_numeric_value_is_error() {
        assert!(parse_duration("abcm").is_err());
        assert!(parse_duration("tensecs").is_err());
    }

    #[test]
    fn single_char_is_error() {
        assert!(parse_duration("m").is_err());
    }
}
