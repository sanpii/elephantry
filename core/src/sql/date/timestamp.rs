impl crate::ToSql for chrono::NaiveDateTime {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::TIMESTAMP
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        self.format("%F %T").to_string().to_sql()
    }
}

impl crate::FromSql for chrono::NaiveDateTime {
    fn from_text(ty: &crate::pq::Type, raw: Option<&str>) -> crate::Result<Self> {
        if let Ok(date) = chrono::NaiveDateTime::parse_from_str(crate::not_null(raw)?, "%F %T") {
            return Ok(date);
        }

        match chrono::NaiveDateTime::parse_from_str(crate::not_null(raw)?, "%F %T.%f") {
            Ok(date) => Ok(date),
            _ => Err(Self::error(ty, raw)),
        }
    }

    fn from_binary(ty: &crate::pq::Type, raw: Option<&[u8]>) -> crate::Result<Self> {
        let t = i64::from_binary(ty, raw)?;
        let base = chrono::NaiveDate::from_ymd(2000, 1, 1).and_hms(0, 0, 0);

        Ok(base + chrono::Duration::microseconds(t))
    }
}

#[cfg(test)]
mod test {
    crate::sql_test!(
        timestamp,
        chrono::NaiveDateTime,
        [(
            "'1970-01-01 00:00:00'",
            chrono::NaiveDateTime::from_timestamp(0, 0),
        )]
    );
}
