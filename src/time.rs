use crate::os::get_time;
use no_std_io::{EndianRead, EndianWrite};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct YearMonthDate {
    pub year: u16,
    pub month: u16,
    pub date: u16,
}

impl Default for YearMonthDate {
    fn default() -> Self {
        Self {
            year: 2000,
            month: 1,
            date: 1,
        }
    }
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
impl YearMonthDate {
    fn new(days_since_epoch: u32) -> Self {
        if days_since_epoch < 60 {
            if days_since_epoch == 0 {
                return Default::default();
            }

            // We can only be in January or February here
            let zero_indexed_month = days_since_epoch as u16 / 32;
            return Self {
                year: 2000,
                month: zero_indexed_month + 1,
                date: (days_since_epoch as u16 % 32) + zero_indexed_month,
            };
        }

        // Remove January and February and start the year with March - we'll
        // add January and February back later.
        // This means we don't have to worry about this year being a leap year,
        // nor do we have to worry about a month with 28 days.
        let adjusted_days = days_since_epoch - 60;

        // 146097 is the number of days in 400 years, including leap days
        let remaining_days_of_400_years = adjusted_days % 146097;
        let number_of_400_years = adjusted_days / 146097;

        // 36524 is the number of days in 100 years, including leap days
        let remaining_days_of_100_years = remaining_days_of_400_years % 36524;
        let number_of_100_years = remaining_days_of_400_years / 36524;

        // 1461 is the number of days in 4 years, including leap day
        let remaining_days_of_4_years = remaining_days_of_100_years % 1461;
        let number_of_4_years = remaining_days_of_100_years / 1461;

        // 365 is the number of days in a non-leap year
        let remaining_days_of_1_year = remaining_days_of_4_years % 365;
        let number_of_1_years = remaining_days_of_4_years / 365;

        let temp_year = number_of_400_years * 400
            + number_of_100_years * 100
            + number_of_4_years * 4
            + number_of_1_years;

        let mut year = temp_year + 2000;
        let mut month = ((remaining_days_of_1_year * 5) + 2) / 153;
        let mut date = (remaining_days_of_1_year - ((month * 153) + 2) / 5) + 1;

        if (number_of_1_years == 4) || (number_of_100_years == 4) {
            month = 2;
            date = 29;
        } else if month < 10 {
            month += 3;
        } else {
            year += 1;
            month = month.checked_sub(9).unwrap_or(1)
        }

        Self {
            year: year as u16,
            month: month as u16,
            date: date as u16,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, EndianRead, EndianWrite)]
pub struct FormattedTimestamp {
    raw: u64,
}

impl Default for FormattedTimestamp {
    fn default() -> Self {
        FormattedTimestamp::new(2020, 1, 1, 0, 0, 0)
    }
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
impl FormattedTimestamp {
    pub fn new(year: u16, month: u16, date: u16, hours: u16, minutes: u16, seconds: u16) -> Self {
        Self {
            raw: ((year as u64) << 26)
                + ((month as u64 & 15) << 22)
                + ((date as u64 & 31) << 17)
                + ((hours as u64 & 31) << 12)
                + ((minutes as u64 & 63) << 6)
                + (seconds as u64 & 63),
        }
    }

    fn get_days_since_system_epoch(&self) -> i32 {
        let year = self.get_year() as i32;

        // The real sysmodule accounts for dates before 2000, even though
        // January 1, 2000 is the 3ds epoch.
        // We're not going to since that can cause issues if someone
        // is intentionally using bad data.
        if year < 2000 {
            return 0;
        }

        let month = self.get_month() as i32;

        // Remove January and February and start the year with March - we'll
        // add January and February back later.
        // This means we don't have to worry about this year being a leap year,
        // nor do we have to worry about a month with 28 days.
        // If the month is before March, pretend it's last year, otherwise
        // continue as if it's the current year.
        let (adjusted_month, adjusted_year) = if month < 3 {
            (month + 9, year - 2001)
        } else {
            (month - 3, year - 2000)
        };

        // 1461 is the number of days in 4 years, including leap day
        // The magic of accounting for leap day happens when we round down during division
        let days_from_last_100_years = ((adjusted_year % 100) * 1461) / 4;
        // 146097 is the number of days in 400 years, including leap days
        // The magic of accounting for leap days happens when we round down during division
        let days_from_over_100_years_ago = ((adjusted_year / 100) * 146097) / 4;

        // Leap days were accounted for above, so we just need the days for this year since March (month 0)
        // Remember, January or February counted as the previous year, which we already accounted for
        // January + February have 59 days when it's not a leap year
        // 365 days - 59 days = 306 remaining days
        // 12 months - 2 months = 10 remaining months
        // 4 and 5 both appear to be valid correction numbers to handle months with 30 days
        // Interestingly, the sysmodule binary actually uses ((adjusted_month * 153) + 2) / 5
        let days_from_current_year_months = ((adjusted_month * 306) + 4) / 10;

        days_from_current_year_months
            + days_from_last_100_years
            + days_from_over_100_years_ago
            + self.get_date() as i32
            // The additional 59 days comes from January and February
            + 59
    }

    pub fn get_year(&self) -> u16 {
        (self.raw >> 26) as u16
    }

    pub fn get_month(&self) -> u16 {
        ((self.raw & 0x3c00000) >> 22) as u16
    }

    pub fn get_date(&self) -> u16 {
        ((self.raw & 0x3e0000) >> 17) as u16
    }

    pub fn get_hours(&self) -> u16 {
        ((self.raw & 0x1f000) >> 12) as u16
    }

    pub fn get_minutes(&self) -> u16 {
        ((self.raw & 0xfc0) >> 6) as u16
    }

    pub fn get_seconds(&self) -> u16 {
        (self.raw & 0x3f) as u16
    }
}

impl From<u64> for FormattedTimestamp {
    fn from(timestamp: u64) -> Self {
        Self { raw: timestamp }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, EndianRead, EndianWrite)]
pub struct SystemTimestamp {
    raw: u64,
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
impl SystemTimestamp {
    pub fn new(millis: u64) -> Self {
        Self { raw: millis }
    }

    pub fn from_unix_timestamp(unix_millis: u64) -> Self {
        Self {
            raw: unix_millis - 946684800000u64,
        }
    }

    pub fn get_unix_timestamp(&self) -> u64 {
        // There's a 30 year offset between the 3ds epoch and the unix epoch
        self.raw + 946684800000u64
    }

    pub fn get_epoch(&self) -> u64 {
        self.raw
    }

    pub fn get_year_month_date(&self) -> YearMonthDate {
        YearMonthDate::new(self.get_days_since_system_epoch())
    }

    pub fn get_days_since_system_epoch(&self) -> u32 {
        (self.raw / 86400000u64) as u32
    }

    pub fn get_hours(&self) -> u16 {
        ((self.raw / 3600000u64) % 24u64) as u16
    }

    pub fn get_minutes(&self) -> u16 {
        ((self.raw / 60000u64) % 60u64) as u16
    }

    pub fn get_seconds(&self) -> u16 {
        ((self.raw / 1000u64) % 60u64) as u16
    }
}

impl From<FormattedTimestamp> for SystemTimestamp {
    fn from(timestamp: FormattedTimestamp) -> Self {
        let seconds_as_ms = timestamp.get_seconds() as u64 * 1000;
        let minutes_as_ms = timestamp.get_minutes() as u64 * 60000;
        let hours_as_ms = timestamp.get_hours() as u64 * 3600000;
        let days_as_ms = timestamp.get_days_since_system_epoch() as u64 * 86400000;

        Self {
            raw: seconds_as_ms + minutes_as_ms + hours_as_ms + days_as_ms,
        }
    }
}

impl From<SystemTimestamp> for FormattedTimestamp {
    fn from(timestamp: SystemTimestamp) -> Self {
        let seconds = timestamp.get_seconds();
        let minutes = timestamp.get_minutes();
        let hours = timestamp.get_hours();
        let days_since_epoch = timestamp.get_days_since_system_epoch();
        let YearMonthDate { year, month, date } = YearMonthDate::new(days_since_epoch);

        Self::new(year, month, date, hours, minutes, seconds)
    }
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn calculate_time_difference_from_now(unix_timestamp: u64) -> u64 {
    unix_timestamp.saturating_sub(get_time())
}

#[cfg(test)]
mod test {
    use super::*;

    mod year_month_date {
        use super::*;

        #[test]
        fn should_get_correct_days() {
            let time: SystemTimestamp = FormattedTimestamp::new(2021, 3, 14, 0, 0, 0).into();
            let days_since_epoch = time.get_days_since_system_epoch();
            let result = YearMonthDate::new(days_since_epoch);
            assert_eq!(
                result,
                YearMonthDate {
                    year: 2021,
                    month: 3,
                    date: 14
                }
            );
        }

        #[test]
        fn should_get_correct_days_on_a_leap_day() {
            let time: SystemTimestamp = FormattedTimestamp::new(2040, 2, 29, 0, 0, 0).into();
            let days_since_epoch = time.get_days_since_system_epoch();
            let result = YearMonthDate::new(days_since_epoch);
            assert_eq!(
                result,
                YearMonthDate {
                    year: 2040,
                    month: 2,
                    date: 29
                }
            );
        }

        #[test]
        fn should_get_correct_days_before_march_2000() {
            let time: SystemTimestamp = FormattedTimestamp::new(2000, 2, 20, 0, 0, 0).into();
            let days_since_epoch = time.get_days_since_system_epoch();
            let result = YearMonthDate::new(days_since_epoch);
            assert_eq!(
                result,
                YearMonthDate {
                    year: 2000,
                    month: 2,
                    date: 20
                }
            );
        }

        #[test]
        fn should_return_the_default_date_when_given_0() {
            let result = YearMonthDate::new(0);
            assert_eq!(result, YearMonthDate::default());
        }

        #[test]
        fn should_not_add_leap_days_every_100_years() {
            let time: SystemTimestamp = FormattedTimestamp::new(2103, 1, 1, 0, 0, 0).into();
            let days_since_epoch = time.get_days_since_system_epoch();
            let result = YearMonthDate::new(days_since_epoch);
            assert_eq!(
                result,
                YearMonthDate {
                    year: 2103,
                    month: 1,
                    date: 1
                }
            );
        }

        #[test]
        fn should_add_a_leap_day_every_400_years() {
            let time: SystemTimestamp = FormattedTimestamp::new(2408, 12, 25, 0, 0, 0).into();
            let days_since_epoch = time.get_days_since_system_epoch();
            let result = YearMonthDate::new(days_since_epoch);
            assert_eq!(
                result,
                YearMonthDate {
                    year: 2408,
                    month: 12,
                    date: 25
                }
            );
        }
    }

    mod system_timestamp {
        use super::*;

        #[test]
        fn get_days_since_system_epoch() {
            let timestamp = SystemTimestamp::new(1640441716000u64);
            let result = timestamp.get_days_since_system_epoch();
            assert_eq!(result, 18986);
        }

        #[test]
        fn get_hours() {
            let timestamp = SystemTimestamp::new(1640441716000u64);
            let result = timestamp.get_hours();
            assert_eq!(result, 14);
        }

        #[test]
        fn get_minutes() {
            let timestamp = SystemTimestamp::new(1640441716000u64);
            let result = timestamp.get_minutes();
            assert_eq!(result, 15);
        }

        #[test]
        fn get_seconds() {
            let timestamp = SystemTimestamp::new(1640441716000u64);
            let result = timestamp.get_seconds();
            assert_eq!(result, 16);
        }

        #[test]
        fn get_year_month_date() {
            let timestamp = SystemTimestamp::new(1640441716000u64);
            let result = timestamp.get_year_month_date();
            assert_eq!(result.year, 2051);
            assert_eq!(result.month, 12);
            assert_eq!(result.date, 25);
        }

        #[test]
        fn from_formatted_timestamp() {
            let formatted_timestamp = FormattedTimestamp::new(2021, 2, 28, 20, 49, 52);
            let system_timestamp = SystemTimestamp::from(formatted_timestamp);
            assert_eq!(system_timestamp.get_unix_timestamp(), 1614545392000);
        }

        #[test]
        fn from_unix_timestamp() {
            let timestamp = SystemTimestamp::from_unix_timestamp(1640441716000u64);
            let YearMonthDate { year, month, date } = timestamp.get_year_month_date();

            assert_eq!(year, 2021);
            assert_eq!(month, 12);
            assert_eq!(date, 25);
            assert_eq!(timestamp.get_hours(), 14);
            assert_eq!(timestamp.get_minutes(), 15);
            assert_eq!(timestamp.get_seconds(), 16);
        }
    }

    mod formatted_timestamp {
        use super::*;

        #[test]
        fn new() {
            let time = FormattedTimestamp::new(2040, 2, 29, 20, 49, 52);
            assert_eq!(time.raw, 0b01111111100000101110110100110001110100);
        }

        #[test]
        fn from_u64() {
            let raw_timestamp = 0b01111111100000101110110100110001110100u64;
            let formatted_timestamp = FormattedTimestamp::from(raw_timestamp);
            assert_eq!(
                formatted_timestamp.raw,
                0b01111111100000101110110100110001110100
            );
        }

        #[test]
        fn get_year() {
            let time = FormattedTimestamp::new(2040, 2, 29, 20, 49, 52);
            assert_eq!(time.get_year(), 2040)
        }

        #[test]
        fn get_month() {
            let time = FormattedTimestamp::new(2040, 2, 29, 20, 49, 52);
            assert_eq!(time.get_month(), 2)
        }

        #[test]
        fn get_date() {
            let time = FormattedTimestamp::new(2040, 2, 29, 20, 49, 52);
            assert_eq!(time.get_date(), 29)
        }

        #[test]
        fn get_hours() {
            let time = FormattedTimestamp::new(2040, 2, 29, 20, 49, 52);
            assert_eq!(time.get_hours(), 20)
        }

        #[test]
        fn get_minutes() {
            let time = FormattedTimestamp::new(2040, 2, 29, 20, 49, 52);
            assert_eq!(time.get_minutes(), 49)
        }

        #[test]
        fn get_seconds() {
            let time = FormattedTimestamp::new(2040, 2, 29, 20, 49, 52);
            assert_eq!(time.get_seconds(), 52)
        }

        mod get_days_since_system_epoch {
            use super::*;

            #[test]
            fn should_get_correct_days() {
                let time = FormattedTimestamp::new(2021, 3, 14, 0, 0, 0);
                assert_eq!(time.get_days_since_system_epoch(), 7743);
            }

            #[test]
            fn should_get_correct_days_on_a_leap_day() {
                let time = FormattedTimestamp::new(2040, 2, 29, 0, 0, 0);
                assert_eq!(time.get_days_since_system_epoch(), 14669);
            }

            #[test]
            fn should_get_correct_days_before_march_2000() {
                let time = FormattedTimestamp::new(2000, 2, 20, 0, 0, 0);
                assert_eq!(time.get_days_since_system_epoch(), 51);
            }

            #[test]
            fn should_return_0_for_years_before_2000() {
                let time = FormattedTimestamp::new(1999, 1, 1, 0, 0, 0);
                assert_eq!(time.get_days_since_system_epoch(), 0);
            }

            #[test]
            fn should_not_add_leap_days_every_100_years() {
                let time = FormattedTimestamp::new(2103, 1, 1, 0, 0, 0);
                assert_eq!(time.get_days_since_system_epoch(), 37620);
            }

            #[test]
            fn should_add_a_leap_day_every_400_years() {
                let time = FormattedTimestamp::new(2408, 12, 25, 0, 0, 0);
                assert_eq!(time.get_days_since_system_epoch(), 149378);
            }

            #[test]
            fn from_system_timestamp() {
                let system_timestamp = SystemTimestamp::from_unix_timestamp(1640433600000u64);
                let formatted_timestamp = FormattedTimestamp::from(system_timestamp);
                let expected_result = FormattedTimestamp::new(2021, 12, 25, 12, 0, 0);
                assert_eq!(formatted_timestamp, expected_result);
            }
        }
    }

    // This is necessary for mocking to work since it can't be a no_std dev dependency
    #[cfg(not(target_os = "horizon"))]
    mod calculate_time_difference_from_now {
        use super::*;
        use crate::os::get_time;
        use mocktopus::mocking::{MockResult, Mockable};

        #[test]
        fn should_subtract_the_current_time_from_the_service_locator_timestamp() {
            get_time.mock_safe(|| MockResult::Return(1u64));

            let result = calculate_time_difference_from_now(3);
            assert_eq!(result, 2);
        }

        #[test]
        fn should_return_0_if_the_current_time_is_before_the_service_locator_timestamp() {
            get_time.mock_safe(|| MockResult::Return(2u64));

            let result = calculate_time_difference_from_now(1);
            assert_eq!(result, 0);
        }
    }
}
