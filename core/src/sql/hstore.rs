use std::collections::HashMap;

/**
 * Rust type for [hstore](https://www.postgresql.org/docs/current/hstore.html).
 */
#[derive(Clone, Debug, PartialEq)]
pub struct Hstore(HashMap<String, Option<String>>);

impl Hstore {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    fn read_string(buf: &mut &[u8]) -> crate::Result<Option<String>> {
        use byteorder::ReadBytesExt;

        let len = buf.read_i32::<byteorder::BigEndian>()?;

        let s = if len < 0 {
            None
        } else {
            let mut vec = Vec::new();
            for _ in 0..len {
                vec.push(buf.read_u8()?);
            }

            Some(String::from_utf8(vec)?)
        };

        Ok(s)
    }
}

impl Default for Hstore {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::DerefMut for Hstore {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::ops::Deref for Hstore {
    type Target = HashMap<String, Option<String>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl crate::ToSql for crate::Hstore {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::TEXT
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        let mut vec = Vec::new();

        for (key, value) in self.iter() {
            let v = value
                .as_ref()
                .map(|x| format!("\"{}\"", x))
                .unwrap_or_else(|| "NULL".to_string());

            vec.push(format!("\"{}\"=>{}", key, v));
        }

        vec.join(", ").to_sql()
    }
}

impl crate::FromSql for Hstore {
    fn from_text(_: &crate::pq::Type, raw: Option<&str>) -> crate::Result<Self> {
        lazy_static::lazy_static! {
            static ref REGEX: regex::Regex = regex::Regex::new(
                "\"(?P<key>.*?)\"=>(\"(?P<value>.*?)\"|(?P<null>NULL))",
            ).unwrap();
        }

        let mut hstore = Self::new();

        for capture in REGEX.captures_iter(crate::not_null(raw)?) {
            let key = capture.name("key").unwrap().as_str().to_string();
            let value = if capture.name("null").is_some() {
                None
            } else {
                Some(capture.name("value").unwrap().as_str().to_string())
            };
            hstore.insert(key, value);
        }

        Ok(hstore)
    }

    /*
     * https://github.com/postgres/postgres/blob/REL_12_0/contrib/hstore/hstore_io.c#L1226
     */
    fn from_binary(ty: &crate::pq::Type, raw: Option<&[u8]>) -> crate::Result<Self> {
        use byteorder::ReadBytesExt;

        let mut hstore = Self::new();
        let mut buf = crate::from_sql::not_null(raw)?;
        let count = buf.read_i32::<byteorder::BigEndian>()?;

        for _ in 0..count {
            let key = Self::read_string(&mut buf)?.ok_or_else(|| Self::error(ty, raw))?;
            let value = Self::read_string(&mut buf)?;

            hstore.insert(key, value);
        }

        Ok(hstore)
    }
}

#[cfg(test)]
mod test {
    crate::sql_test!(
        hstore,
        crate::Hstore,
        [("'a=>1, b => 2, c=>null'", {
            let mut hstore = crate::Hstore::new();
            hstore.insert("a".to_string(), Some("1".to_string()));
            hstore.insert("b".to_string(), Some("2".to_string()));
            hstore.insert("c".to_string(), None);

            hstore
        })]
    );
}
