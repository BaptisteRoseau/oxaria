//! HTML's valid date, time and duration strings, which the `time` role's
//! text is limited to.

/// Any of HTML's date, time, date-time, week or duration forms.
pub fn is_valid(text: &str) -> bool {
    is_year(text)
        || is_month(text)
        || is_date(text)
        || is_yearless_date(text)
        || is_week(text)
        || is_time(text)
        || is_date_time(text)
        || is_time_zone(text)
        || is_iso_duration(text)
        || is_component_duration(text)
}

fn digits(text: &str, count: usize) -> Option<u32> {
    (text.len() == count && text.chars().all(|c| c.is_ascii_digit()))
        .then(|| text.parse().ok())
        .flatten()
}

fn in_range(value: Option<u32>, min: u32, max: u32) -> bool {
    value.is_some_and(|value| (min..=max).contains(&value))
}

fn is_year(text: &str) -> bool {
    text.len() >= 4
        && text.chars().all(|c| c.is_ascii_digit())
        && !text.trim_start_matches('0').is_empty()
}

fn is_month(text: &str) -> bool {
    text.rsplit_once('-')
        .is_some_and(|(year, month)| is_year(year) && in_range(digits(month, 2), 1, 12))
}

fn is_date(text: &str) -> bool {
    text.rsplit_once('-')
        .is_some_and(|(month, day)| is_month(month) && in_range(digits(day, 2), 1, 31))
}

fn is_yearless_date(text: &str) -> bool {
    let text = text.strip_prefix("--").unwrap_or(text);
    text.split_once('-').is_some_and(|(month, day)| {
        in_range(digits(month, 2), 1, 12) && in_range(digits(day, 2), 1, 31)
    })
}

fn is_week(text: &str) -> bool {
    text.split_once("-W")
        .is_some_and(|(year, week)| is_year(year) && in_range(digits(week, 2), 1, 53))
}

fn is_time(text: &str) -> bool {
    let mut parts = text.split(':');
    let (Some(hours), Some(minutes)) = (parts.next(), parts.next()) else {
        return false;
    };
    let seconds_are_valid = match parts.next() {
        None => true,
        Some(seconds) => is_seconds(seconds),
    };
    parts.next().is_none()
        && in_range(digits(hours, 2), 0, 23)
        && in_range(digits(minutes, 2), 0, 59)
        && seconds_are_valid
}

fn is_seconds(text: &str) -> bool {
    let (whole, fraction) = text.split_once('.').unwrap_or((text, "1"));
    in_range(digits(whole, 2), 0, 59)
        && (1..=3).contains(&fraction.len())
        && fraction.chars().all(|c| c.is_ascii_digit())
}

fn is_date_time(text: &str) -> bool {
    text.split_once(['T', ' '])
        .is_some_and(|(date, time)| is_date(date) && (is_time(time) || is_zoned_time(time)))
}

fn is_zoned_time(text: &str) -> bool {
    match text.strip_suffix('Z') {
        Some(time) => is_time(time),
        None => text
            .rfind(['+', '-'])
            .is_some_and(|split| is_time(&text[..split]) && is_time_zone(&text[split..])),
    }
}

fn is_time_zone(text: &str) -> bool {
    let offset = match text {
        "Z" => return true,
        _ => text.strip_prefix(['+', '-']),
    };
    offset.is_some_and(|offset| {
        let (hours, minutes) = offset
            .split_once(':')
            .unwrap_or_else(|| offset.split_at(offset.len().min(2)));
        in_range(digits(hours, 2), 0, 23) && in_range(digits(minutes, 2), 0, 59)
    })
}

/// `P1DT4H18M3S`.
fn is_iso_duration(text: &str) -> bool {
    let Some(rest) = text.strip_prefix('P') else {
        return false;
    };
    let (days, time) = rest.split_once('T').unwrap_or((rest, ""));
    let days_are_valid = days.is_empty() || days.strip_suffix('D').is_some_and(is_number);
    let has_time = rest.contains('T');
    days_are_valid
        && (!has_time || (!time.is_empty() && is_iso_time_components(time)))
        && rest != "T"
        && !rest.is_empty()
}

fn is_iso_time_components(mut text: &str) -> bool {
    for unit in ['H', 'M', 'S'] {
        if let Some((value, rest)) = text.split_once(unit) {
            let is_valid = match unit {
                'S' => is_decimal(value),
                _ => is_number(value),
            };
            if !is_valid {
                return false;
            }
            text = rest;
        }
    }
    text.is_empty()
}

/// `4h 18m 3s`, each unit at most once.
fn is_component_duration(text: &str) -> bool {
    let components: Vec<&str> = text.split_ascii_whitespace().collect();
    let mut units = Vec::new();
    let all_valid = components.iter().all(|component| {
        let Some(unit) = component.chars().last().filter(|u| "wdhms".contains(*u)) else {
            return false;
        };
        let value = &component[..component.len() - 1];
        let is_valid = match unit {
            's' => is_decimal(value),
            _ => is_number(value),
        };
        let is_new = !units.contains(&unit);
        units.push(unit);
        is_valid && is_new
    });
    !components.is_empty() && all_valid
}

fn is_number(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| c.is_ascii_digit())
}

fn is_decimal(text: &str) -> bool {
    match text.split_once('.') {
        Some((whole, fraction)) => {
            is_number(whole) && (1..=3).contains(&fraction.len()) && is_number(fraction)
        }
        None => is_number(text),
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("2019")]
    #[case("2019-11")]
    #[case("2019-11-18")]
    #[case("11-18")]
    #[case("--11-18")]
    #[case("2019-W47")]
    #[case("09:54")]
    #[case("09:54:39")]
    #[case("09:54:39.123")]
    #[case("2019-11-18T09:54")]
    #[case("2019-11-18 09:54:39Z")]
    #[case("2019-11-18T09:54:39+01:00")]
    #[case("2019-11-18T09:54:39-0100")]
    #[case("+01:00")]
    #[case("4h 18m 3s")]
    #[case("3.5s")]
    #[case("P2D")]
    #[case("PT4H18M3S")]
    #[case("P1DT2H")]
    fn valid_strings(#[case] text: &str) {
        assert!(is_valid(text), "{text}");
    }

    #[rstest]
    #[case("Sept 27")]
    #[case("yesterday")]
    #[case("2019-13-01")]
    #[case("25:00")]
    #[case("9:54")]
    #[case("4h 4h")]
    #[case("P")]
    #[case("PT")]
    #[case("0000")]
    #[case("")]
    fn invalid_strings(#[case] text: &str) {
        assert!(!is_valid(text), "{text}");
    }
}
