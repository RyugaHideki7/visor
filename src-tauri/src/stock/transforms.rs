use chrono::Local;

pub(crate) fn apply_split(value: &str, part: &str) -> String {
    if let Some((before, after)) = value.split_once('+') {
        if part == "before" {
            return before.trim().chars().take(10).collect();
        }
        return after.trim().chars().take(10).collect();
    }
    if part == "before" {
        value.trim().chars().take(10).collect()
    } else {
        "".to_string()
    }
}

pub(crate) fn apply_transformation(value: String, transformation: &str) -> String {
    match transformation {
        "date" => {
            let v = value.trim();
            if v.is_empty() {
                return Local::now().format("%d/%m/%Y").to_string();
            }

            if v.chars().all(|c| c.is_ascii_digit()) && v.len() >= 8 {
                if let Ok(dt) = chrono::NaiveDate::parse_from_str(&v[0..8], "%Y%m%d") {
                    return dt.format("%d/%m/%Y").to_string();
                }
            }

            let formats = [
                "%d/%m/%Y",
                "%Y-%m-%d",
                "%d-%m-%Y",
                "%d.%m.%Y",
                "%d/%m/%y",
                "%d-%m-%y",
                "%d.%m.%y",
                "%Y%m%d",
            ];

            for fmt in formats {
                if let Ok(dt) = chrono::NaiveDate::parse_from_str(v, fmt) {
                    return dt.format("%d/%m/%Y").to_string();
                }
            }
            Local::now().format("%d/%m/%Y").to_string()
        }
        "heure" => {
            let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
            if digits.len() >= 14 {
                digits[8..14].to_string()
            } else if digits.len() >= 12 {
                format!("{}00", &digits[8..12])
            } else if digits.len() >= 6 {
                digits[..6].to_string()
            } else if digits.len() >= 4 {
                format!("{}00", &digits[..4])
            } else {
                "000000".to_string()
            }
        }
        "datetime" => {
            let v = value.trim();
            if v.is_empty() {
                return Local::now().format("%d/%m/%Y %H:%M:%S").to_string();
            }

            if v.chars().all(|c| c.is_ascii_digit()) {
                if v.len() >= 14 {
                    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&v[..14], "%Y%m%d%H%M%S") {
                        return dt.format("%d/%m/%Y %H:%M:%S").to_string();
                    }
                }
                if v.len() >= 8 {
                    if let Ok(d) = chrono::NaiveDate::parse_from_str(&v[..8], "%Y%m%d") {
                        let t_str = if v.len() >= 14 { &v[8..14] } else { "000000" };
                        if let Ok(t) = chrono::NaiveTime::parse_from_str(t_str, "%H%M%S") {
                            return chrono::NaiveDateTime::new(d, t)
                                .format("%d/%m/%Y %H:%M:%S")
                                .to_string();
                        }
                        return d.and_hms_opt(0, 0, 0)
                            .unwrap()
                            .format("%d/%m/%Y %H:%M:%S")
                            .to_string();
                    }
                }
            }

            let formats = [
                "%d/%m/%Y %H:%M:%S",
                "%d/%m/%Y %H:%M",
                "%Y-%m-%d %H:%M:%S",
                "%Y-%m-%d %H:%M",
                "%Y%m%d %H%M%S",
                "%Y%m%d%H%M%S",
                "%d/%m/%Y",
                "%Y-%m-%d",
                "%Y%m%d",
            ];

            for fmt in formats {
                if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(v, fmt) {
                    return dt.format("%d/%m/%Y %H:%M:%S").to_string();
                }
                if let Ok(d) = chrono::NaiveDate::parse_from_str(v, fmt) {
                    let dt = d
                        .and_hms_opt(0, 0, 0)
                        .unwrap_or_else(|| {
                            chrono::NaiveDateTime::new(
                                d,
                                chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
                            )
                        });
                    return dt.format("%d/%m/%Y %H:%M:%S").to_string();
                }
            }

            Local::now().format("%d/%m/%Y %H:%M:%S").to_string()
        }
        "decimal" => {
            let cleaned = value.replace(',', ".");
            cleaned
                .chars()
                .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                .collect::<String>()
        }
        "tinyint" => {
            let n = value.trim().parse::<i64>().unwrap_or(1);
            if n == 2 { "2".to_string() } else { "1".to_string() }
        }
        "current_datetime" => Local::now().format("%d/%m/%Y %H:%M:%S").to_string(),
        "datetime_combine" => {
            let parts: Vec<&str> = value.split(';').collect();
            if parts.len() < 2 {
                return Local::now().format("%d/%m/%Y %H:%M:%S").to_string();
            }
            let date_part = parts[0].trim();
            let time_part = parts[1].trim();

            let date_formats = ["%d/%m/%Y", "%Y-%m-%d", "%d-%m-%Y", "%d.%m.%Y", "%Y%m%d"];
            let time_formats = ["%H:%M:%S", "%H.%M.%S", "%H%M%S", "%H:%M", "%H.%M"];

            let mut date_obj: Option<chrono::NaiveDate> = None;
            let mut time_obj: Option<chrono::NaiveTime> = None;
            for fmt in date_formats {
                if let Ok(d) = chrono::NaiveDate::parse_from_str(date_part, fmt) {
                    date_obj = Some(d);
                    break;
                }
            }
            for fmt in time_formats {
                if let Ok(t) = chrono::NaiveTime::parse_from_str(time_part, fmt) {
                    time_obj = Some(t);
                    break;
                }
            }

            if let (Some(d), Some(t)) = (date_obj, time_obj) {
                chrono::NaiveDateTime::new(d, t)
                    .format("%d/%m/%Y %H:%M:%S")
                    .to_string()
            } else {
                Local::now().format("%d/%m/%Y %H:%M:%S").to_string()
            }
        }
        _ => value,
    }
}
