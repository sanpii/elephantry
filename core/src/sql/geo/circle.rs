#[derive(Clone, Debug, PartialEq)]
pub struct Circle {
    pub x: f64,
    pub y: f64,
    pub r: f64,
}

impl Circle {
    pub fn new(x: f64, y: f64, r: f64) -> Self {
        Self { x, y, r }
    }
}

impl std::fmt::Display for Circle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}, {}", self.x, self.y, self.r)
    }
}

impl crate::ToSql for Circle {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::CIRCLE
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        self.to_string().to_sql()
    }
}

impl crate::FromSql for Circle {
    fn from_text(ty: &crate::pq::Type, raw: Option<&str>) -> crate::Result<Self> {
        lazy_static::lazy_static! {
            static ref REGEX: regex::Regex = regex::Regex::new(r"([\d\.]+)").unwrap();
        }

        let mut matches = REGEX.find_iter(crate::not_null(raw)?);
        let x =
            crate::Coordinates::coordinate(&matches.next()).ok_or_else(|| Self::error(ty, raw))?;
        let y =
            crate::Coordinates::coordinate(&matches.next()).ok_or_else(|| Self::error(ty, raw))?;
        let r =
            crate::Coordinates::coordinate(&matches.next()).ok_or_else(|| Self::error(ty, raw))?;

        Ok(Self::new(x, y, r))
    }

    /*
     * https://github.com/postgres/postgres/blob/REL_12_0/src/backend/utils/adt/geo_ops.c#L4603
     */
    fn from_binary(_: &crate::pq::Type, raw: Option<&[u8]>) -> crate::Result<Self> {
        use byteorder::ReadBytesExt;

        let mut buf = crate::from_sql::not_null(raw)?;
        let x = buf.read_f64::<byteorder::BigEndian>()?;
        let y = buf.read_f64::<byteorder::BigEndian>()?;
        let r = buf.read_f64::<byteorder::BigEndian>()?;

        Ok(Self::new(x, y, r))
    }
}

#[cfg(test)]
mod test {
    crate::sql_test!(
        circle,
        crate::Circle,
        [("'0, 0, 5'", crate::Circle::new(0., 0., 5.))]
    );
}
