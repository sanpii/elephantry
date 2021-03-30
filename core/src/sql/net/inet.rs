impl crate::ToSql for std::net::IpAddr {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::INET
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        self.to_string().to_sql()
    }
}

impl crate::FromSql for std::net::IpAddr {
    fn from_text(ty: &crate::pq::Type, raw: Option<&str>) -> crate::Result<Self> {
        crate::not_null(raw)?
            .parse()
            .map_err(|_| Self::error(ty, raw))
    }

    /*
     * https://github.com/postgres/postgres/blob/REL_12_0/src/backend/utils/adt/network.c#L267
     */
    fn from_binary(ty: &crate::pq::Type, raw: Option<&[u8]>) -> crate::Result<Self> {
        use std::convert::TryFrom;

        let network = super::Network::try_from(crate::from_sql::not_null(raw)?)?;

        if network.is_cidr {
            return Err(Self::error(ty, raw));
        }

        let ip = match network.ip_familly {
            super::IpFamilly::Inet => {
                let ipv4 = std::net::Ipv4Addr::from(network.ip as u32);
                ipv4.into()
            }
            super::IpFamilly::Inet6 => {
                let ipv6 = std::net::Ipv6Addr::from(network.ip);
                ipv6.into()
            }
        };

        Ok(ip)
    }
}

#[cfg(test)]
mod test {
    crate::sql_test!(
        inet,
        std::net::IpAddr,
        [
            (
                "'127.0.0.1'",
                std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
            ),
            (
                "'127.0.0.1/32'",
                std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
            ),
            ("'::1'", std::net::IpAddr::V6(std::net::Ipv6Addr::LOCALHOST)),
        ]
    );
}
