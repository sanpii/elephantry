/**
 * Trait to allow a rust type to be translated to a SQL value.
 */
pub trait ToSql {
    /** The corresponding SQL type */
    fn ty(&self) -> crate::pq::Type;
    /** Convert the value */
    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>>;

    fn format(&self) -> crate::pq::Format {
        crate::pq::Format::Text
    }

    fn error(&self, message: Option<&String>) -> crate::Error {
        crate::Error::ToSql {
            pg_type: self.ty(),
            rust_type: std::any::type_name::<Self>().to_string(),
            message: message.cloned(),
        }
    }
}

impl ToSql for bool {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::BOOL
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        let v = if *self { b"t\0" } else { b"f\0" };

        Ok(Some(v.to_vec()))
    }
}

impl ToSql for f32 {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::FLOAT4
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        self.to_string().to_sql()
    }
}

impl ToSql for f64 {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::FLOAT8
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        self.to_string().to_sql()
    }
}

impl ToSql for &str {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::VARCHAR
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        let mut v = self.as_bytes().to_vec();
        v.push(0);

        Ok(Some(v))
    }
}

impl ToSql for char {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::BPCHAR
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        self.to_string().to_sql()
    }
}

impl ToSql for String {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::VARCHAR
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        self.as_str().to_sql()
    }
}

impl ToSql for i16 {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::INT2
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        self.to_string().to_sql()
    }
}

impl ToSql for i32 {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::INT4
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        self.to_string().to_sql()
    }
}

impl ToSql for i64 {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::INT8
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        self.to_string().to_sql()
    }
}

impl ToSql for u32 {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::INT8
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        self.to_string().to_sql()
    }
}

impl<T: ToSql> ToSql for Option<T> {
    fn ty(&self) -> crate::pq::Type {
        match self {
            Some(data) => data.ty(),
            None => crate::pq::types::UNKNOWN,
        }
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        match self {
            Some(data) => T::to_sql(data),
            None => Ok(None),
        }
    }
}

impl<T: ToSql> ToSql for Vec<T> {
    fn ty(&self) -> crate::pq::Type {
        use crate::pq::ToArray;

        match self.get(0) {
            Some(data) => data.ty().to_array(),
            None => crate::pq::types::UNKNOWN,
        }
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        let mut data = vec![b'{'];

        for x in self {
            let element = match x.to_sql()? {
                Some(element) => element,
                None => b"null\0".to_vec(),
            };

            data.extend_from_slice(&element[..element.len() - 1]);
            data.push(b',');
        }

        if data.last() == Some(&b',') {
            data.pop();
        }

        data.extend_from_slice(b"}\0");

        Ok(Some(data))
    }
}

impl ToSql for () {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::UNKNOWN
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        Ok(None)
    }
}

#[cfg(test)]
mod test {
    use crate::ToSql;

    #[test]
    fn vec_to_sql() {
        let vec = vec![1, 2, 3];

        assert_eq!(vec.to_sql().unwrap(), Some(b"{1,2,3}\0".to_vec()));
    }

    #[test]
    fn empty_vec() {
        let vec = Vec::<String>::new();

        assert_eq!(vec.to_sql().unwrap(), Some(b"{}\0".to_vec()));
    }
}
