#[derive(Clone, Debug, PartialEq)]
pub struct Path(geo_types::LineString<f64>);

impl Path {
    pub fn new(coordinates: &crate::Coordinates) -> Self {
        Self(geo_types::LineString(
            coordinates.iter().map(|x| *x.clone()).collect(),
        ))
    }
}

impl std::ops::Deref for Path {
    type Target = geo_types::LineString<f64>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        for coordinate in self.0.points_iter() {
            s.push_str(&format!("({}, {}),", coordinate.x(), coordinate.y()));
        }

        write!(f, "{}", s.trim_end_matches(','))
    }
}

impl crate::ToSql for Path {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::PATH
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        self.to_string().to_sql()
    }
}

impl crate::FromSql for Path {
    fn from_text(ty: &crate::pq::Type, raw: Option<&str>) -> crate::Result<Self> {
        let coordinates = crate::not_null(raw)?
            .parse()
            .map_err(|_| Self::error(ty, raw))?;

        Ok(Self::new(&coordinates))
    }

    /*
     * https://github.com/postgres/postgres/blob/REL_12_0/src/backend/utils/adt/geo_ops.c#L1485
     */
    fn from_binary(_: &crate::pq::Type, raw: Option<&[u8]>) -> crate::Result<Self> {
        use byteorder::ReadBytesExt;

        let mut buf = crate::not_null(raw)?;
        let _closed = buf.read_u8()?;
        let npts = buf.read_i32::<byteorder::BigEndian>()?;
        let mut coordinates = Vec::new();

        for _ in 0..npts {
            let x = buf.read_f64::<byteorder::BigEndian>()?;
            let y = buf.read_f64::<byteorder::BigEndian>()?;

            let coordinate = crate::Coordinate::new(x, y);
            coordinates.push(coordinate);
        }

        Ok(Self::new(&coordinates.into()))
    }
}

#[cfg(test)]
mod test {
    crate::sql_test!(
        path,
        crate::Path,
        [
            (
                "'((0, 0), (10, 10), (10, 0), (0, 0))'",
                crate::Path::new(
                    &vec![
                        crate::Coordinate::new(0., 0.),
                        crate::Coordinate::new(10., 10.),
                        crate::Coordinate::new(10., 0.),
                        crate::Coordinate::new(0., 0.),
                    ]
                    .into()
                )
            ),
            (
                "'[(0, 0), (10, 10), (10, 0)]'",
                crate::Path::new(
                    &vec![
                        crate::Coordinate::new(0., 0.),
                        crate::Coordinate::new(10., 10.),
                        crate::Coordinate::new(10., 0.),
                    ]
                    .into()
                )
            ),
        ]
    );
}
