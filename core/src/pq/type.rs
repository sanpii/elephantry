pub use libpq::types;

pub type Type = libpq::Type;

use std::collections::HashMap;

impl crate::FromSql for Type {
    fn from_binary(ty: &Type, raw: Option<&[u8]>) -> crate::Result<Self> {
        String::from_binary(ty, raw)?
            .parse()
            .map_err(|_| Self::error(ty, raw))
    }

    fn from_text(ty: &Type, raw: Option<&str>) -> crate::Result<Self> {
        crate::not_null(raw)?
            .parse()
            .map_err(|_| Self::error(ty, raw))
    }
}

impl crate::ToSql for Type {
    fn ty(&self) -> Type {
        types::TEXT
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        self.name.to_sql()
    }
}

lazy_static::lazy_static! {
    static ref TYPES: HashMap<&'static str, &'static str> = {
        use std::any::type_name as t;

        let mut types = HashMap::new();
        types.insert(types::BIT.name, t::<u8>());
        types.insert(types::BOOL.name, t::<bool>());
        types.insert(types::CHAR.name, t::<char>());
        types.insert(types::FLOAT4.name, t::<f32>());
        types.insert(types::FLOAT8.name, t::<f64>());
        types.insert(types::INT2.name, t::<i16>());
        types.insert(types::INT4.name, t::<i32>());
        types.insert(types::INT8.name, t::<i64>());
        types.insert(types::TEXT.name, t::<String>());
        types.insert(types::VARCHAR.name, t::<String>());

        types.insert(
            types::BYTEA.name,
            #[cfg(feature = "bit")]
            t::<crate::Bytea>(),
            #[cfg(not(feature = "bit"))]
            "elephantry::Bytea",
        );
        types.insert(
            types::VARBIT.name,
            #[cfg(feature = "bit")]
            t::<bit_vec::BitVec>(),
            #[cfg(not(feature = "bit"))]
            "bit_vec::BitVec",
        );

        types.insert(
            types::DATE.name,
            #[cfg(feature = "date")]
            t::<chrono::NaiveDate>(),
            #[cfg(not(feature = "date"))]
            "chrono::NaiveDate",
        );

        types.insert(
            types::BOX.name,
            #[cfg(feature = "geo")]
            t::<crate::Box>(),
            #[cfg(not(feature = "geo"))]
            "elephantry::Box",
        );
        types.insert(
            types::CIRCLE.name,
            #[cfg(feature = "geo")]
            t::<crate::Circle>(),
            #[cfg(not(feature = "geo"))]
            "elephantry::Circle",
        );
        types.insert(
            types::LINE.name,
            #[cfg(feature = "geo")]
            t::<crate::Line>(),
            #[cfg(not(feature = "geo"))]
            "elephantry::Line",
        );
        types.insert(
            types::LSEG.name,
            #[cfg(feature = "geo")]
            t::<crate::Segment>(),
            #[cfg(not(feature = "geo"))]
            "elephantry::Segment",
        );
        types.insert(
            types::PATH.name,
            #[cfg(feature = "geo")]
            t::<crate::Path>(),
            #[cfg(not(feature = "geo"))]
            "elephantry::Path",
        );
        types.insert(
            types::POINT.name,
            #[cfg(feature = "geo")]
            t::<crate::Point>(),
            #[cfg(not(feature = "geo"))]
            "elephantry::Point",
        );
        types.insert(
            types::POLYGON.name,
            #[cfg(feature = "geo")]
            t::<crate::Polygon>(),
            #[cfg(not(feature = "geo"))]
            "elephantry::Polygon",
        );

        types.insert(
            types::CIDR.name,
            #[cfg(feature = "net")]
            t::<ipnetwork::IpNetwork>(),
            #[cfg(not(feature = "net"))]
            "ipnetwork::IpNetwork",
        );
        types.insert(
            types::INET.name,
            #[cfg(feature = "net")]
            t::<std::net::IpAddr>(),
            #[cfg(not(feature = "net"))]
            "std::net::IpAddr",
        );
        types.insert(
            types::MACADDR.name,
            #[cfg(feature = "net")]
            t::<macaddr::MacAddr6>(),
            #[cfg(not(feature = "net"))]
            "macaddr::MacAddr6",
        );
        types.insert(
            types::MACADDR8.name,
            #[cfg(feature = "net")]
            t::<macaddr::MacAddr8>(),
            #[cfg(not(feature = "net"))]
            "macaddr::MacAddr8",
        );

        types.insert(
            types::JSON.name,
            #[cfg(feature = "json")]
            t::<serde_json::Value>(),
            #[cfg(not(feature = "json"))]
            "serde_json::Value",
        );
        types.insert(
            types::JSONB.name,
            #[cfg(feature = "json")]
            t::<serde_json::Value>(),
            #[cfg(not(feature = "json"))]
            "serde_json::Value",
        );

        types.insert(
            types::MONEY.name,
            #[cfg(feature = "money")]
            t::<postgres_money::Money>(),
            #[cfg(not(feature = "money"))]
            "postgres_money::Money",
        );

        types.insert(
            types::NUMERIC.name,
            #[cfg(feature = "numeric")]
            t::<bigdecimal::BigDecimal>(),
            #[cfg(not(feature = "numeric"))]
            "bigdecimal::BigDecimal",
        );

        types.insert(
            types::TIME.name,
            #[cfg(feature = "time")]
            t::<chrono::NaiveTime>(),
            #[cfg(not(feature = "time"))]
            "chrono::NaiveTime",
        );
        types.insert(
            types::TIMETZ.name,
            #[cfg(feature = "time")]
            t::<crate::TimeTz>(),
            #[cfg(not(feature = "time"))]
            "chrono::TimeTz",
        );
        types.insert(
            types::TIMESTAMP.name,
            #[cfg(feature = "time")]
            t::<chrono::NaiveDateTime>(),
            #[cfg(not(feature = "time"))]
            "chrono::NaiveDateTime",
        );
        types.insert(
            types::TIMESTAMPTZ.name,
            #[cfg(feature = "time")]
            t::<chrono::DateTime<chrono::FixedOffset>>(),
            #[cfg(not(feature = "time"))]
            "chrono::DateTime<chrono::FixedOffset>>",
        );

        types.insert(
            types::UUID.name,
            #[cfg(feature = "uuid")]
            t::<uuid::Uuid>(),
            #[cfg(not(feature = "uuid"))]
            "uuid::Uuid",
        );

        types.insert(
            types::XML.name,
            #[cfg(feature = "xml")]
            t::<xmltree::Element>(),
            #[cfg(not(feature = "xml"))]
            "xmltree::Element",
        );

        types
    };
}

#[doc(hidden)]
pub fn sql_to_rust(ty: &crate::pq::Type) -> String {
    let rty = TYPES.get(ty.name).unwrap_or(&"String");

    if matches!(ty.kind, crate::pq::types::Kind::Array(_)) {
        format!("Vec<{}>", rty)
    } else {
        rty.to_string()
    }
}

pub(crate) trait ToArray {
    fn to_array(&self) -> Self;
}

impl ToArray for Type {
    fn to_array(&self) -> Self {
        match *self {
            types::ACLITEM => types::ACLITEM_ARRAY,
            types::BIT => types::BIT_ARRAY,
            types::BOOL => types::BOOL_ARRAY,
            types::BOX => types::BOX_ARRAY,
            types::BPCHAR => types::BPCHAR_ARRAY,
            types::BYTEA => types::BYTEA_ARRAY,
            types::CHAR => types::CHAR_ARRAY,
            types::CID => types::CID_ARRAY,
            types::CIDR => types::CIDR_ARRAY,
            types::CIRCLE => types::CIRCLE_ARRAY,
            types::CSTRING => types::CSTRING_ARRAY,
            types::DATE => types::DATE_ARRAY,
            types::DATE_RANGE => types::DATE_RANGE_ARRAY,
            types::FLOAT4 => types::FLOAT4_ARRAY,
            types::FLOAT8 => types::FLOAT8_ARRAY,
            types::GTS_VECTOR => types::GTS_VECTOR_ARRAY,
            types::INET => types::INET_ARRAY,
            types::INT2 => types::INT2_ARRAY,
            types::INT2_VECTOR => types::INT2_VECTOR_ARRAY,
            types::INT4 => types::INT4_ARRAY,
            types::INT4_RANGE => types::INT4_RANGE_ARRAY,
            types::INT8 => types::INT8_ARRAY,
            types::INT8_RANGE => types::INT8_RANGE_ARRAY,
            types::INTERVAL => types::INTERVAL_ARRAY,
            types::JSON => types::JSON_ARRAY,
            types::JSONB => types::JSONB_ARRAY,
            types::JSONPATH => types::JSONPATH_ARRAY,
            types::LINE => types::LINE_ARRAY,
            types::LSEG => types::LSEG_ARRAY,
            types::MACADDR => types::MACADDR_ARRAY,
            types::MACADDR8 => types::MACADDR8_ARRAY,
            types::MONEY => types::MONEY_ARRAY,
            types::NAME => types::NAME_ARRAY,
            types::NUMERIC => types::NUMERIC_ARRAY,
            types::NUM_RANGE => types::NUM_RANGE_ARRAY,
            types::OID => types::OID_ARRAY,
            types::OID_VECTOR => types::OID_VECTOR_ARRAY,
            types::PATH => types::PATH_ARRAY,
            types::PG_LSN => types::PG_LSN_ARRAY,
            types::POINT => types::POINT_ARRAY,
            types::POLYGON => types::POLYGON_ARRAY,
            types::RECORD => types::RECORD_ARRAY,
            types::REFCURSOR => types::REFCURSOR_ARRAY,
            types::REGCLASS => types::REGCLASS_ARRAY,
            types::REGCONFIG => types::REGCONFIG_ARRAY,
            types::REGDICTIONARY => types::REGDICTIONARY_ARRAY,
            types::REGNAMESPACE => types::REGNAMESPACE_ARRAY,
            types::REGOPER => types::REGOPER_ARRAY,
            types::REGOPERATOR => types::REGOPERATOR_ARRAY,
            types::REGPROC => types::REGPROC_ARRAY,
            types::REGPROCEDURE => types::REGPROCEDURE_ARRAY,
            types::REGROLE => types::REGROLE_ARRAY,
            types::REGTYPE => types::REGTYPE_ARRAY,
            types::TEXT => types::TEXT_ARRAY,
            types::TID => types::TID_ARRAY,
            types::TIMESTAMP => types::TIMESTAMP_ARRAY,
            types::TIMESTAMPTZ => types::TIMESTAMPTZ_ARRAY,
            types::TIME => types::TIME_ARRAY,
            types::TIMETZ => types::TIMETZ_ARRAY,
            types::TSQUERY => types::TSQUERY_ARRAY,
            types::TSTZ_RANGE => types::TSTZ_RANGE_ARRAY,
            types::TS_RANGE => types::TS_RANGE_ARRAY,
            types::TS_VECTOR => types::TS_VECTOR_ARRAY,
            types::TXID_SNAPSHOT => types::TXID_SNAPSHOT_ARRAY,
            types::UUID => types::UUID_ARRAY,
            types::VARBIT => types::VARBIT_ARRAY,
            types::VARCHAR => types::VARCHAR_ARRAY,
            types::XID => types::XID_ARRAY,
            types::XML => types::XML_ARRAY,
            _ => self.clone(),
        }
    }
}
