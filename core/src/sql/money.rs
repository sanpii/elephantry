pub use postgres_money::Money;

impl crate::ToSql for Money {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::MONEY
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        self.to_string().to_sql()
    }
}

impl crate::FromSql for Money {
    fn from_text(ty: &crate::pq::Type, raw: Option<&str>) -> crate::Result<Self> {
        let s = String::from_text(ty, raw)?;

        Self::parse_str(&s).map_err(|_| Self::error(ty, raw))
    }

    /*
     * https://github.com/postgres/postgres/blob/REL_12_0/src/backend/utils/adt/cash.c#L524
     */
    fn from_binary(ty: &crate::pq::Type, raw: Option<&[u8]>) -> crate::Result<Self> {
        let cents = i64::from_binary(ty, raw)?;

        Ok(Self::from(cents))
    }
}

#[cfg(test)]
mod test {
    crate::sql_test!(money, crate::Money, [("1.00", crate::Money::from(100))]);
}
