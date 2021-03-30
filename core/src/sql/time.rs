pub use time::Time;
pub use time::UtcOffset as Timezone;
pub type TimeTz = (Time, Timezone);

impl crate::ToSql for Time {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::TIME
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        self.to_string().to_sql()
    }
}

impl crate::FromSql for Time {
    fn from_text(ty: &crate::pq::Type, raw: Option<&str>) -> crate::Result<Self> {
        Time::parse(crate::not_null(raw)?, "%T").map_err(|_| Self::error(ty, raw))
    }

    /*
     * https://github.com/postgres/postgres/blob/REL_12_0/src/backend/utils/adt/date.c#L1281
     */
    fn from_binary(ty: &crate::pq::Type, raw: Option<&[u8]>) -> crate::Result<Self> {
        let usec = i64::from_binary(ty, raw)?;

        Ok(Time::midnight() + time::Duration::microseconds(usec))
    }
}

impl crate::ToSql for TimeTz {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::TIMETZ
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        format!("{}{}", self.0, self.1).to_sql()
    }
}

impl crate::FromSql for TimeTz {
    fn from_text(ty: &crate::pq::Type, raw: Option<&str>) -> crate::Result<Self> {
        let value = crate::not_null(raw)?;
        dbg!(&value);

        let x = match value.find(|c| c == '+' || c == '-') {
            Some(x) => x,
            None => return Err(Self::error(ty, raw)),
        };

        let time = Time::parse(&value[0..x], "%T").map_err(|err| {
            dbg!(err);
            Self::error(ty, raw)
        })?;

        let mut tz = value[x..].replace(':', "");

        if tz.len() == 3 {
            tz.push_str("00");
        }

        let timezone = Timezone::parse(&tz, "%z").map_err(|err| {
            dbg!(err);
            Self::error(ty, raw)
        })?;

        Ok((time, timezone))
    }

    /*
     * https://github.com/postgres/postgres/blob/REL_12_0/src/backend/utils/adt/date.c#L2063
     */
    fn from_binary(_: &crate::pq::Type, raw: Option<&[u8]>) -> crate::Result<Self> {
        use byteorder::ReadBytesExt;

        let mut buf = crate::from_sql::not_null(raw)?;
        let time = buf.read_i64::<byteorder::BigEndian>()?;
        let zone = buf.read_i32::<byteorder::BigEndian>()?;

        Ok((
            Time::midnight() + time::Duration::microseconds(time),
            Timezone::seconds(-zone),
        ))
    }
}

#[cfg(test)]
mod test {
    crate::sql_test!(
        time,
        crate::Time,
        [
            ("'00:00:00'", crate::Time::midnight()),
            ("'01:02:03'", time::time!(01:02:03)),
        ]
    );

    crate::sql_test!(
        timetz,
        crate::TimeTz,
        [
            (
                "'00:00:00+0000'",
                (crate::Time::midnight(), crate::Timezone::UTC)
            ),
            (
                "'01:02:03+0200'",
                (time::time!(01:02:03), time::offset!(+2))
            ),
        ]
    );
}
