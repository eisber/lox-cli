//! clock module — see lib.rs for architecture overview
//!
//! Simulated time with sunrise/sunset calculation.

use std::f64::consts::PI;

/// Simulated clock for the SPS simulator.
///
/// Tracks wall-clock time starting from a given date/time and advancing
/// in discrete steps. Provides calendar queries and simplified solar
/// position calculations for sunrise/sunset.
#[derive(Debug, Clone)]
pub struct SimClock {
    /// Unix timestamp of the simulation start.
    epoch_secs: f64,
    /// Total elapsed simulation time in seconds.
    elapsed_secs: f64,
    /// Latitude for sunrise/sunset (default: 48.2, Vienna).
    latitude: f64,
    /// Longitude for sunrise/sunset (default: 16.4, Vienna).
    longitude: f64,
}

/// Days in each month (non-leap).
const DAYS_IN_MONTH: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

fn is_leap_year(y: u32) -> bool {
    (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400)
}

fn days_in_month(y: u32, m: u32) -> u32 {
    if m == 2 && is_leap_year(y) {
        29
    } else {
        DAYS_IN_MONTH[(m - 1) as usize]
    }
}

/// Convert a date/time to a Unix timestamp (UTC, no leap seconds).
fn datetime_to_unix(year: u32, month: u32, day: u32, hour: u32, min: u32) -> f64 {
    // Days from epoch 1970-01-01
    let mut days: i64 = 0;
    // Years
    for y in 1970..year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }
    // Months in current year
    for m in 1..month {
        days += days_in_month(year, m) as i64;
    }
    // Days in current month (1-indexed)
    days += (day - 1) as i64;

    days as f64 * 86400.0 + hour as f64 * 3600.0 + min as f64 * 60.0
}

/// Break a Unix timestamp back into (year, month, day, hour, min, sec).
fn unix_to_datetime(ts: f64) -> (u32, u32, u32, u32, u32, u32) {
    let total_secs = ts.floor() as i64;
    let mut remaining = total_secs;

    let secs_in_day: i64 = 86400;
    let mut days = remaining / secs_in_day;
    let day_secs = remaining % secs_in_day;
    // Handle negative day_secs
    let day_secs = if day_secs < 0 {
        days -= 1;
        day_secs + secs_in_day
    } else {
        day_secs
    };
    remaining = day_secs;

    let hour = (remaining / 3600) as u32;
    remaining %= 3600;
    let minute = (remaining / 60) as u32;
    let second = (remaining % 60) as u32;

    // Days since 1970-01-01
    let mut year: u32 = 1970;
    loop {
        let days_in_year: i64 = if is_leap_year(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }

    let mut month: u32 = 1;
    loop {
        let dim = days_in_month(year, month) as i64;
        if days < dim {
            break;
        }
        days -= dim;
        month += 1;
    }
    let day = days as u32 + 1;

    (year, month, day, hour, minute, second)
}

impl SimClock {
    /// Create a new clock at the given date and time (UTC).
    pub fn new(year: u32, month: u32, day: u32, hour: u32, min: u32) -> Self {
        Self {
            epoch_secs: datetime_to_unix(year, month, day, hour, min),
            elapsed_secs: 0.0,
            latitude: 48.2,
            longitude: 16.4,
        }
    }

    /// Set the geographic location for sunrise/sunset calculations.
    pub fn set_location(&mut self, latitude: f64, longitude: f64) {
        self.latitude = latitude;
        self.longitude = longitude;
    }

    /// Advance the simulation clock by `dt` seconds.
    pub fn advance(&mut self, dt: f64) {
        self.elapsed_secs += dt;
    }

    /// Current Unix timestamp.
    fn current_ts(&self) -> f64 {
        self.epoch_secs + self.elapsed_secs
    }

    /// Current date/time decomposed.
    fn current_datetime(&self) -> (u32, u32, u32, u32, u32, u32) {
        unix_to_datetime(self.current_ts())
    }

    /// Minutes elapsed since midnight (0.0 – 1440.0).
    pub fn minutes_since_midnight(&self) -> f64 {
        let (_, _, _, h, m, s) = self.current_datetime();
        h as f64 * 60.0 + m as f64 + s as f64 / 60.0
    }

    /// Current hour (0–23).
    pub fn hour(&self) -> u32 {
        self.current_datetime().3
    }

    /// Current minute (0–59).
    pub fn minute(&self) -> u32 {
        self.current_datetime().4
    }

    /// Current second (0–59).
    pub fn second(&self) -> u32 {
        self.current_datetime().5
    }

    /// Day of week: 0 = Monday, 6 = Sunday.
    ///
    /// 1970-01-01 was a Thursday (index 3).
    pub fn day_of_week(&self) -> u32 {
        let ts = self.current_ts();
        let days_since_epoch = (ts / 86400.0).floor() as i64;
        // 1970-01-01 = Thursday = 3
        ((days_since_epoch % 7 + 3) % 7) as u32
    }

    /// Day of year (1-indexed, Jan 1 = 1).
    pub fn day_of_year(&self) -> u32 {
        let (year, month, day, _, _, _) = self.current_datetime();
        let mut doy = day;
        for m in 1..month {
            doy += days_in_month(year, m);
        }
        doy
    }

    /// Current month (1–12).
    pub fn month(&self) -> u32 {
        self.current_datetime().1
    }

    /// Sunrise time as minutes since midnight, using simplified solar calculation.
    ///
    /// Uses the "sunrise equation" with a simple declination model:
    ///   δ = -23.44° × cos(2π/365 × (day_of_year + 10))
    ///   cos(ω₀) = (sin(-0.833°) - sin(lat) × sin(δ)) / (cos(lat) × cos(δ))
    ///   sunrise = 720 - 4×(longitude + degrees(ω₀)) + eqtime
    ///
    /// Equation of time is approximated as:
    ///   eot = 9.87 × sin(2B) - 7.53 × cos(B) - 1.5 × sin(B)
    ///   where B = 2π/365 × (day_of_year - 81)
    pub fn sunrise_minutes(&self) -> f64 {
        self.solar_event(true)
    }

    /// Sunset time as minutes since midnight.
    pub fn sunset_minutes(&self) -> f64 {
        self.solar_event(false)
    }

    /// Whether it's currently nighttime (before sunrise or after sunset).
    pub fn is_nighttime(&self) -> bool {
        let now = self.minutes_since_midnight();
        now < self.sunrise_minutes() || now > self.sunset_minutes()
    }

    /// Core sunrise/sunset calculation.
    fn solar_event(&self, is_sunrise: bool) -> f64 {
        let doy = self.day_of_year() as f64;
        let lat = self.latitude;
        let lng = self.longitude;

        // Solar declination (degrees)
        let decl_rad = -23.44_f64.to_radians() * ((2.0 * PI / 365.0) * (doy + 10.0)).cos();

        // Hour angle at sunrise/sunset
        // Standard solar elevation at sunrise/sunset: -0.833° (accounting for refraction)
        let lat_rad = lat.to_radians();
        let cos_omega = ((-0.833_f64).to_radians().sin() - lat_rad.sin() * decl_rad.sin())
            / (lat_rad.cos() * decl_rad.cos());

        // Clamp for polar day/night
        let cos_omega = cos_omega.clamp(-1.0, 1.0);
        let omega_deg = cos_omega.acos().to_degrees();

        // Equation of time (minutes) — Spencer approximation
        let b = (2.0 * PI / 365.0) * (doy - 81.0);
        let eot = 9.87 * (2.0 * b).sin() - 7.53 * b.cos() - 1.5 * b.sin();

        // Solar noon in minutes (UTC)
        let solar_noon = 720.0 - 4.0 * lng - eot;

        if is_sunrise {
            solar_noon - 4.0 * omega_deg
        } else {
            solar_noon + 4.0 * omega_deg
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_clock() {
        let c = SimClock::new(2025, 6, 21, 12, 0);
        assert_eq!(c.hour(), 12);
        assert_eq!(c.minute(), 0);
        assert_eq!(c.second(), 0);
        assert_eq!(c.month(), 6);
    }

    #[test]
    fn test_advance() {
        let mut c = SimClock::new(2025, 1, 1, 23, 59);
        assert_eq!(c.hour(), 23);
        assert_eq!(c.minute(), 59);
        c.advance(120.0); // 2 minutes
        assert_eq!(c.hour(), 0);
        assert_eq!(c.minute(), 1);
        // Day rolled over
        assert_eq!(c.current_datetime().2, 2); // day = 2
    }

    #[test]
    fn test_minutes_since_midnight() {
        let c = SimClock::new(2025, 3, 15, 14, 30);
        let msm = c.minutes_since_midnight();
        assert!((msm - 870.0).abs() < 0.01); // 14*60 + 30 = 870
    }

    #[test]
    fn test_day_of_week() {
        // 2025-01-06 is a Monday
        let c = SimClock::new(2025, 1, 6, 12, 0);
        assert_eq!(c.day_of_week(), 0); // Monday

        // 2025-01-05 is a Sunday
        let c = SimClock::new(2025, 1, 5, 12, 0);
        assert_eq!(c.day_of_week(), 6); // Sunday

        // 1970-01-01 was a Thursday
        let c = SimClock::new(1970, 1, 1, 0, 0);
        assert_eq!(c.day_of_week(), 3); // Thursday
    }

    #[test]
    fn test_day_of_year() {
        let c = SimClock::new(2025, 1, 1, 0, 0);
        assert_eq!(c.day_of_year(), 1);

        let c = SimClock::new(2025, 12, 31, 0, 0);
        assert_eq!(c.day_of_year(), 365);

        // Leap year
        let c = SimClock::new(2024, 12, 31, 0, 0);
        assert_eq!(c.day_of_year(), 366);
    }

    #[test]
    fn test_sunrise_sunset_vienna_summer() {
        // Vienna, 2025-06-21 (summer solstice) — longest day
        let c = SimClock::new(2025, 6, 21, 12, 0);
        let sr = c.sunrise_minutes();
        let ss = c.sunset_minutes();

        // Sunrise should be roughly 3:50–5:00 UTC (Vienna is UTC+1/+2)
        // In UTC: ~3:50 = 230 min, but with our simplified model allow wider range
        assert!(
            sr > 150.0 && sr < 330.0,
            "sunrise={sr} unexpected for Vienna summer"
        );
        assert!(
            ss > 1100.0 && ss < 1300.0,
            "sunset={ss} unexpected for Vienna summer"
        );

        // Day length should be ~15.5–16.5 hours
        let day_len = (ss - sr) / 60.0;
        assert!(day_len > 14.0 && day_len < 18.0, "day_length={day_len}h");
    }

    #[test]
    fn test_sunrise_sunset_vienna_winter() {
        // Vienna, 2025-12-21 (winter solstice) — shortest day
        let c = SimClock::new(2025, 12, 21, 12, 0);
        let sr = c.sunrise_minutes();
        let ss = c.sunset_minutes();

        // Day length should be ~8–9 hours
        let day_len = (ss - sr) / 60.0;
        assert!(day_len > 7.0 && day_len < 10.0, "day_length={day_len}h");
    }

    #[test]
    fn test_is_nighttime() {
        // Midnight — should be nighttime
        let c = SimClock::new(2025, 6, 21, 1, 0);
        assert!(c.is_nighttime());

        // Noon — should be daytime
        let c = SimClock::new(2025, 6, 21, 12, 0);
        assert!(!c.is_nighttime());
    }

    #[test]
    fn test_equinox_roughly_equal_day_night() {
        // March equinox — day and night should be roughly equal
        let c = SimClock::new(2025, 3, 20, 12, 0);
        let sr = c.sunrise_minutes();
        let ss = c.sunset_minutes();
        let day_len = (ss - sr) / 60.0;
        assert!(
            (day_len - 12.0).abs() < 1.5,
            "equinox day_length={day_len}h, expected ~12h"
        );
    }

    #[test]
    fn test_datetime_roundtrip() {
        // Verify our conversion is consistent
        let ts = datetime_to_unix(2025, 7, 15, 8, 30);
        let (y, m, d, h, mi, s) = unix_to_datetime(ts);
        assert_eq!((y, m, d, h, mi, s), (2025, 7, 15, 8, 30, 0));
    }

    #[test]
    fn test_leap_year() {
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2025));
    }
}
