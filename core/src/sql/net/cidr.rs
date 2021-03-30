impl crate::ToSql for ipnetwork::IpNetwork {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::CIDR
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        self.to_string().to_sql()
    }
}

impl crate::FromSql for ipnetwork::IpNetwork {
    fn from_text(ty: &crate::pq::Type, raw: Option<&str>) -> crate::Result<Self> {
        crate::not_null(raw)?
            .parse()
            .map_err(|_| Self::error(ty, raw))
    }

    /*
     * https://github.com/postgres/postgres/blob/REL_12_0/src/backend/utils/adt/network.c#L275
     */
    fn from_binary(ty: &crate::pq::Type, raw: Option<&[u8]>) -> crate::Result<Self> {
        use std::convert::TryFrom;

        let network = super::Network::try_from(crate::from_sql::not_null(raw)?)?;

        if !network.is_cidr {
            return Err(Self::error(ty, raw));
        }

        let cidr = match network.ip_familly {
            super::IpFamilly::Inet => {
                let cidr =
                    ipnetwork::Ipv4Network::new((network.ip as u32).into(), network.netmask_bits)
                        .map_err(|_| Self::error(ty, raw))?;
                ipnetwork::IpNetwork::V4(cidr)
            }
            super::IpFamilly::Inet6 => {
                let cidr = ipnetwork::Ipv6Network::new(network.ip.into(), network.netmask_bits)
                    .map_err(|_| Self::error(ty, raw))?;
                ipnetwork::IpNetwork::V6(cidr)
            }
        };

        Ok(cidr)
    }
}

#[cfg(test)]
mod test {
    crate::sql_test!(
        cidr,
        ipnetwork::IpNetwork,
        [(
            "'192.168.1.0/24'",
            ipnetwork::IpNetwork::V4("192.168.1.0/24".parse().unwrap())
        )]
    );
}
