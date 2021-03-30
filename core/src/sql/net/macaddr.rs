impl crate::ToSql for macaddr::MacAddr6 {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::MACADDR
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        self.to_string().to_sql()
    }
}

impl crate::FromSql for macaddr::MacAddr6 {
    fn from_text(ty: &crate::pq::Type, raw: Option<&str>) -> crate::Result<Self> {
        crate::not_null(raw)?
            .parse()
            .map_err(|_| Self::error(ty, raw))
    }

    /*
     * https://github.com/postgres/postgres/blob/REL_12_0/src/backend/utils/adt/mac.c#L161
     */
    fn from_binary(_: &crate::pq::Type, raw: Option<&[u8]>) -> crate::Result<Self> {
        use byteorder::ReadBytesExt;

        let mut buf = crate::from_sql::not_null(raw)?;
        let mut parts = [0; 6];
        for part in &mut parts {
            *part = buf.read_u8()?;
        }

        Ok(parts.into())
    }
}

#[cfg(test)]
mod test {
    #![allow(non_snake_case)]

    static MAC: macaddr::MacAddr6 = macaddr::MacAddr6::new(0x08, 0x00, 0x2b, 0x01, 0x02, 0x03);

    crate::sql_test!(
        Macaddr,
        macaddr::MacAddr6,
        [
            ("'08:00:2b:01:02:03'", crate::sql::net::macaddr::test::MAC),
            ("'08-00-2b-01-02-03'", crate::sql::net::macaddr::test::MAC),
            ("'08002b:010203'", crate::sql::net::macaddr::test::MAC),
            ("'08002b-010203'", crate::sql::net::macaddr::test::MAC),
            ("'0800.2b01.0203'", crate::sql::net::macaddr::test::MAC),
            ("'0800-2b01-0203'", crate::sql::net::macaddr::test::MAC),
            ("'08002b010203'", crate::sql::net::macaddr::test::MAC),
        ]
    );
}
