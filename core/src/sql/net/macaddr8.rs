impl crate::ToSql for macaddr::MacAddr8 {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::MACADDR8
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        self.to_string().to_sql()
    }
}

impl crate::FromSql for macaddr::MacAddr8 {
    fn from_text(ty: &crate::pq::Type, raw: Option<&str>) -> crate::Result<Self> {
        crate::not_null(raw)?
            .parse()
            .map_err(|_| Self::error(ty, raw))
    }

    /*
     * https://github.com/postgres/postgres/blob/REL_12_0/src/backend/utils/adt/mac8.c#L295
     */
    fn from_binary(_: &crate::pq::Type, raw: Option<&[u8]>) -> crate::Result<Self> {
        use byteorder::ReadBytesExt;

        let mut buf = crate::from_sql::not_null(raw)?;
        let mut parts = [0; 8];
        for part in &mut parts {
            *part = buf.read_u8()?;
        }

        Ok(parts.into())
    }
}

#[cfg(test)]
mod test {
    #![allow(non_snake_case)]

    static MAC: macaddr::MacAddr8 =
        macaddr::MacAddr8::new(0x08, 0x00, 0x2b, 0x01, 0x02, 0x03, 0x04, 0x05);

    crate::sql_test!(
        macaddr8,
        macaddr::MacAddr8,
        [
            (
                "'08:00:2b:01:02:03:04:05'",
                crate::sql::net::macaddr8::test::MAC
            ),
            (
                "'08-00-2b-01-02-03-04-05'",
                crate::sql::net::macaddr8::test::MAC
            ),
            ("'08002b:0102030405'", crate::sql::net::macaddr8::test::MAC),
            ("'08002b-0102030405'", crate::sql::net::macaddr8::test::MAC),
            (
                "'0800.2b01.0203.0405'",
                crate::sql::net::macaddr8::test::MAC
            ),
            (
                "'0800-2b01-0203-0405'",
                crate::sql::net::macaddr8::test::MAC
            ),
            ("'08002b0102030405'", crate::sql::net::macaddr8::test::MAC),
        ]
    );
}
