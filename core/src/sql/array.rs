/*
 * https://github.com/postgres/postgres/blob/REL_12_0/src/include/utils/array.h
 */

use byteorder::ReadBytesExt;
use std::convert::TryInto;

/**
 * Rust type for [array](https://www.postgresql.org/docs/current/arrays.html).
 */
#[derive(Clone, Debug)]
pub struct Array<T> {
    ndim: usize,
    elemtype: crate::pq::Type,
    has_nulls: bool,
    dimensions: Vec<i32>,
    lower_bounds: Vec<i32>,
    data: Vec<u8>,
    maker: std::marker::PhantomData<T>,
}

impl<T: crate::FromSql> Iterator for Array<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        use bytes::Buf;

        if self.data.is_empty() {
            return None;
        }

        let mut buf = &self.data.clone()[..];

        let mut len = buf.get_u32() as usize;
        let value = if len == 0xFFFF_FFFF {
            len = 0;
            None
        } else {
            Some(&buf[..len])
        };
        self.data = buf[len..].to_vec();

        match T::from_sql(&self.elemtype, crate::pq::Format::Binary, value) {
            Ok(x) => Some(x),
            Err(err) => {
                log::error!("Unable to convert array element from SQL: {}", err);
                None
            }
        }
    }
}

impl<T: crate::FromSql> crate::FromSql for Array<T> {
    fn from_text(_ty: &crate::pq::Type, _raw: Option<&str>) -> crate::Result<Self> {
        todo!()
    }

    fn from_binary(_: &crate::pq::Type, raw: Option<&[u8]>) -> crate::Result<Self> {
        let mut data = crate::not_null(raw)?;

        let ndim = data.read_i32::<byteorder::BigEndian>()?;
        if ndim < 0 {
            panic!("Invalid array");
        }

        let has_nulls = data.read_i32::<byteorder::BigEndian>()? != 0;

        let oid = data.read_u32::<byteorder::BigEndian>()?;
        let elemtype: crate::pq::Type = oid.try_into().unwrap_or(crate::pq::Type {
            oid,
            descr: "Custom type",
            name: "custom",
            kind: libpq::types::Kind::Composite,
        });

        let mut dimensions = Vec::new();
        let mut lower_bounds = Vec::new();

        for _ in 0..ndim {
            let dimension = data.read_i32::<byteorder::BigEndian>()?;
            dimensions.push(dimension);

            let lower_bound = data.read_i32::<byteorder::BigEndian>()?;
            lower_bounds.push(lower_bound);
        }

        let array = Self {
            ndim: ndim as usize,
            elemtype,
            has_nulls,
            dimensions,
            lower_bounds,
            data: data.to_vec(),
            maker: std::marker::PhantomData,
        };

        Ok(array)
    }
}

impl<T: crate::FromSql> From<Array<T>> for Vec<T> {
    fn from(array: Array<T>) -> Self {
        if array.ndim > 1 {
            panic!(
                "Unable to transform {} dimension array as vector",
                array.ndim
            );
        }

        array.collect()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn bin_vec() -> crate::Result {
        let elephantry = crate::test::new_conn()?;
        let results: Vec<i32> = elephantry.query_one("SELECT '{1, 2}'::int4[]", &[])?;

        assert_eq!(results, vec![1, 2]);

        Ok(())
    }

    #[test]
    fn bin_array_str() -> crate::Result {
        let elephantry = crate::test::new_conn()?;
        let results: Vec<Option<String>> =
            elephantry.query_one("SELECT '{null, str}'::text[]", &[])?;

        assert_eq!(results, vec![None, Some("str".to_string())]);

        Ok(())
    }
}
