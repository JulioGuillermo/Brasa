//! Seconds as m:ss, or h:mm:ss from one hour.

pub fn clock(seconds: f64) -> String {
    let seconds = seconds.max(0.0) as u64;
    if seconds >= 3600 {
        return format!("{}:{:02}:{:02}", seconds / 3600, seconds / 60 % 60, seconds % 60);
    }
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_minutes_and_hours() {
        assert_eq!(clock(0.0), "0:00");
        assert_eq!(clock(59.9), "0:59");
        assert_eq!(clock(259.0), "4:19");
        assert_eq!(clock(3600.0), "1:00:00");
        assert_eq!(clock(3725.0), "1:02:05");
        assert_eq!(clock(-5.0), "0:00");
    }
}
