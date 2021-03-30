#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Interval {
    pub years: i32,
    pub months: i32,
    pub days: i32,
    pub hours: i32,
    pub mins: i32,
    pub secs: i32,
    pub usecs: i32,
}

impl Interval {
    pub fn new(
        years: i32,
        months: i32,
        days: i32,
        hours: i32,
        mins: i32,
        secs: i32,
        usecs: i32,
    ) -> Self {
        Self {
            years,
            months,
            days,
            hours,
            mins,
            secs,
            usecs,
        }
    }

    pub fn year() -> Self {
        Self::years(1)
    }

    pub fn years(n: i32) -> Self {
        Self::new(n, 0, 0, 0, 0, 0, 0)
    }

    pub fn month() -> Self {
        Self::months(1)
    }

    pub fn months(n: i32) -> Self {
        Self::new(0, n, 0, 0, 0, 0, 0)
    }

    pub fn day() -> Self {
        Self::days(1)
    }

    pub fn days(n: i32) -> Self {
        Self::new(0, 0, n, 0, 0, 0, 0)
    }

    pub fn hour() -> Self {
        Self::hours(1)
    }

    pub fn hours(n: i32) -> Self {
        Self::new(0, 0, 0, n, 0, 0, 0)
    }

    pub fn minute() -> Self {
        Self::minutes(1)
    }

    pub fn minutes(n: i32) -> Self {
        Self::new(0, 0, 0, 0, n, 0, 0)
    }

    pub fn second() -> Self {
        Self::seconds(1)
    }

    pub fn seconds(n: i32) -> Self {
        Self::new(0, 0, 0, 0, 0, n, 0)
    }

    pub fn microsecond() -> Self {
        Self::microseconds(1)
    }

    pub fn microseconds(n: i32) -> Self {
        Self::new(0, 0, 0, 0, 0, 0, n)
    }
}

impl Default for Interval {
    fn default() -> Self {
        Self {
            years: 0,
            months: 0,
            days: 0,
            hours: 0,
            mins: 0,
            secs: 0,
            usecs: 0,
        }
    }
}

impl From<&Interval> for i64 {
    fn from(interval: &Interval) -> Self {
        interval.years as i64 * 12 * 30 * 24 * 60 * 60 * 1_000_000
            + interval.months as i64 * 30 * 24 * 60 * 60 * 1_000_000
            + interval.days as i64 * 24 * 60 * 60 * 1_000_000
            + interval.hours as i64 * 60 * 60 * 1_000_000
            + interval.mins as i64 * 60 * 1_000_000
            + interval.secs as i64 * 1_000_000
            + interval.usecs as i64
    }
}

impl PartialEq for Interval {
    fn eq(&self, other: &Self) -> bool {
        let a: i64 = i64::from(self);
        let b: i64 = i64::from(other);

        a.eq(&b)
    }
}

impl PartialOrd for Interval {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let a: i64 = self.into();

        a.partial_cmp(&other.into())
    }
}

impl std::fmt::Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} years {} months {} days {}:{}:{}.{}",
            self.years, self.months, self.days, self.hours, self.mins, self.secs, self.usecs,
        )
    }
}

macro_rules! caps {
    ($caps:ident, $part:ident, $ty:ident, $raw:ident) => {
        match $caps.name(stringify!($part)) {
            Some(part) => match part.as_str().parse() {
                Ok(part) => part,
                Err(_) => return Err(Self::error($ty, $raw)),
            },
            None => 0,
        };
    };
}

impl crate::FromSql for Interval {
    fn from_text(ty: &crate::pq::Type, raw: Option<&str>) -> crate::Result<Self> {
        lazy_static::lazy_static! {
            static ref REGEX: regex::Regex = regex::Regex::new(
                r"((?P<years>\d+) years?)? ?((?P<months>\d+) (months?|mons?))? ?((?P<days>\d+) days?)? ?((?P<hours>\d+):(?P<mins>\d+):(?P<secs>\d+))?(\.(?P<usecs>\d+))?",
            )
            .unwrap();
        }

        let s = String::from_text(ty, raw)?;

        if s.as_str() == "00:00:00" {
            return Ok(Self::default());
        }

        let caps = match REGEX.captures(&s) {
            Some(caps) => caps,
            None => return Err(Self::error(ty, raw)),
        };

        let years = caps!(caps, years, ty, raw);
        let months = caps!(caps, months, ty, raw);
        let days = caps!(caps, days, ty, raw);
        let hours = caps!(caps, hours, ty, raw);
        let mins = caps!(caps, mins, ty, raw);
        let secs = caps!(caps, secs, ty, raw);
        let usecs = caps!(caps, usecs, ty, raw);

        let interval = Self::new(years, months, days, hours, mins, secs, usecs);

        Ok(interval)
    }

    /*
     * https://github.com/postgres/postgres/blob/REL_12_0/src/backend/utils/adt/timestamp.c#L994
     */
    fn from_binary(_: &crate::pq::Type, raw: Option<&[u8]>) -> crate::Result<Self> {
        use byteorder::ReadBytesExt;

        let mut buf = crate::not_null(raw)?;
        let mut usecs = buf.read_i64::<byteorder::BigEndian>()?;
        let days = buf.read_i32::<byteorder::BigEndian>()?;
        let mut months = buf.read_i32::<byteorder::BigEndian>()?;

        let years = months / 12;
        months %= 12;

        let hours = usecs / (60 * 60 * 1_000_000);
        usecs %= 60 * 60 * 1_000_000;

        let minutes = usecs / (60 * 1_000_000);
        usecs %= 60 * 1_000_000;

        let secs = usecs / 1_000_000;
        usecs %= 1_000_000;

        let interval = Self::new(
            years,
            months,
            days,
            hours as i32,
            minutes as i32,
            secs as i32,
            usecs as i32,
        );

        Ok(interval)
    }
}

impl crate::ToSql for Interval {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::INTERVAL
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        self.to_string().to_sql()
    }
}

#[cfg(test)]
mod test {
    use crate::FromSql;

    #[test]
    fn from_text() {
        let tests = vec![
            ("00:00:00", crate::Interval::new(0, 0, 0, 0, 0, 0, 0)),
            ("1 year", crate::Interval::new(0, 12, 0, 0, 0, 0, 0)),
            ("1 years", crate::Interval::new(1, 0, 0, 0, 0, 0, 0)),
            ("1 month", crate::Interval::new(0, 1, 0, 0, 0, 0, 0)),
            ("1 year 10 days", crate::Interval::new(1, 0, 10, 0, 0, 0, 0)),
            (
                "1 year 2 months 3 days 04:05:06.000007",
                crate::Interval::new(1, 2, 3, 4, 5, 6, 7),
            ),
        ];

        for (value, expected) in tests {
            assert_eq!(
                crate::Interval::from_text(&crate::pq::types::INTERVAL, Some(&value)).unwrap(),
                expected,
            );
        }
    }

    #[test]
    fn from_binary() {
        let tests = vec![
            (
                [0, 0, 0, 3, 48, 151, 149, 151, 0, 0, 0, 0, 0, 0, 0, 0],
                crate::Interval::new(0, 0, 0, 3, 48, 20, 142487),
            ),
            (
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 3, 50],
                crate::Interval::new(68, 2, 4, 0, 0, 0, 0),
            ),
            (
                [0, 0, 0, 3, 108, 139, 192, 128, 0, 0, 0, 3, 0, 0, 0, 14],
                crate::Interval::new(1, 2, 3, 4, 5, 6, 0),
            ),
        ];

        for (value, expected) in tests {
            assert_eq!(
                crate::Interval::from_binary(&crate::pq::types::INTERVAL, Some(&value),).unwrap(),
                expected,
            );
        }
    }
}
