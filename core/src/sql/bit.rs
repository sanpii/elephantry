impl crate::ToSql for u8 {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::BIT
    }

    fn format(&self) -> crate::pq::Format {
        crate::pq::Format::Binary
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        let bytes = bit_vec::BitVec::from_bytes(&[*self]);

        bytes.to_sql()
    }
}

impl crate::FromSql for u8 {
    fn from_text(ty: &crate::pq::Type, raw: Option<&str>) -> crate::Result<Self> {
        let bytes = bit_vec::BitVec::from_text(ty, raw)?;

        bytes
            .get(0)
            .map(|x| x as u8)
            .ok_or_else(|| Self::error(ty, raw))
    }

    /*
     * https://github.com/postgres/postgres/blob/REL_12_0/src/backend/utils/adt/varbit.c#L375
     */
    fn from_binary(ty: &crate::pq::Type, raw: Option<&[u8]>) -> crate::Result<Self> {
        let bytes = bit_vec::BitVec::from_binary(ty, raw)?;

        bytes
            .get(0)
            .map(|x| x as u8)
            .ok_or_else(|| Self::error(ty, raw))
    }
}

impl crate::ToSql for bit_vec::BitVec {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::VARBIT
    }

    fn format(&self) -> crate::pq::Format {
        crate::pq::Format::Binary
    }

    /*
     * https://github.com/postgres/postgres/blob/REL_12_0/src/backend/utils/adt/varbit.c#L680
     */
    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        use bytes::BufMut;

        let mut buf = bytes::BytesMut::new();

        buf.put_u32(self.len() as u32);
        for byte in self.to_bytes() {
            buf.put_u8(byte);
        }

        Ok(Some(buf.to_vec()))
    }
}

impl crate::FromSql for bit_vec::BitVec {
    fn from_text(_: &crate::pq::Type, raw: Option<&str>) -> crate::Result<Self> {
        let s = crate::not_null(raw)?;
        let mut bits = bit_vec::BitVec::from_elem(s.len(), false);

        for (x, bit) in s.chars().enumerate() {
            if bit == '1' {
                bits.set(x, true);
            }
        }

        Ok(bits)
    }

    fn from_binary(_: &crate::pq::Type, raw: Option<&[u8]>) -> crate::Result<Self> {
        use byteorder::ReadBytesExt;

        let mut buf = crate::not_null(raw)?;
        let _size = buf.read_i32::<byteorder::BigEndian>()?;

        Ok(bit_vec::BitVec::from_bytes(buf))
    }
}

#[cfg(test)]
mod test {
    crate::sql_test!(bit, u8, [("'0'", 0), ("'1'", 1), ("0", 0), ("1", 1)]);

    crate::sql_test!(
        varbit,
        bit_vec::BitVec,
        [
            ("'00000000'", bit_vec::BitVec::from_bytes(&[0b00000000])),
            ("'10101010'", bit_vec::BitVec::from_bytes(&[0b10101010])),
            ("'11111111'", bit_vec::BitVec::from_bytes(&[0b11111111])),
        ]
    );
}
