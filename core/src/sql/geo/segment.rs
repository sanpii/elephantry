#[derive(Clone, Debug, PartialEq)]
pub struct Segment(geo_types::Line<f64>);

impl Segment {
    pub fn new(start: crate::Coordinate, end: crate::Coordinate) -> Self {
        Self(geo_types::Line::new(*start, *end))
    }
}

impl std::ops::Deref for Segment {
    type Target = geo_types::Line<f64>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(({}, {}), ({}, {}))",
            self.0.start.x, self.0.start.y, self.0.end.x, self.0.end.y
        )
    }
}

impl crate::ToSql for Segment {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::LSEG
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        self.to_string().to_sql()
    }
}

impl crate::FromSql for Segment {
    fn from_text(ty: &crate::pq::Type, raw: Option<&str>) -> crate::Result<Self> {
        let coordinates = crate::not_null(raw)?
            .parse::<crate::Coordinates>()
            .map_err(|_| Self::error(ty, raw))?;

        if coordinates.len() != 2 {
            return Err(Self::error(ty, raw));
        }

        Ok(Self::new(coordinates[0].clone(), coordinates[1].clone()))
    }

    /*
     * https://github.com/postgres/postgres/blob/REL_12_0/src/backend/utils/adt/geo_ops.c#L2064
     */
    fn from_binary(_: &crate::pq::Type, raw: Option<&[u8]>) -> crate::Result<Self> {
        use byteorder::ReadBytesExt;

        let mut buf = crate::from_sql::not_null(raw)?;
        let start = crate::Coordinate::new(
            buf.read_f64::<byteorder::BigEndian>()?,
            buf.read_f64::<byteorder::BigEndian>()?,
        );
        let end = crate::Coordinate::new(
            buf.read_f64::<byteorder::BigEndian>()?,
            buf.read_f64::<byteorder::BigEndian>()?,
        );

        Ok(Self::new(start, end))
    }
}

#[cfg(test)]
mod test {
    crate::sql_test!(
        lseg,
        crate::Segment,
        [
            (
                "'[(1, 2), (3, 4)]'",
                crate::Segment::new(
                    crate::Coordinate::new(1., 2.),
                    crate::Coordinate::new(3., 4.)
                )
            ),
            (
                "'((10.3, 20.0), (0.5, 0.003))'",
                crate::Segment::new(
                    crate::Coordinate::new(10.3, 20.),
                    crate::Coordinate::new(0.5, 0.003)
                )
            ),
        ]
    );
}
