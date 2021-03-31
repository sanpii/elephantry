impl crate::ToSql for xmltree::Element {
    fn ty(&self) -> crate::pq::Type {
        crate::pq::types::XML
    }

    fn to_sql(&self) -> crate::Result<Option<Vec<u8>>> {
        let mut vec = Vec::new();

        self.write(&mut vec)
            .map_err(|e| self.error(Some(&e.to_string())))?;
        vec.push(b'\0');

        Ok(Some(vec))
    }
}

impl crate::FromSql for xmltree::Element {
    fn from_text(ty: &crate::pq::Type, raw: Option<&str>) -> crate::Result<Self> {
        xmltree::Element::parse(crate::not_null(raw)?.as_bytes())
            .map_err(|_| Self::error(ty, raw))
    }

    /*
     * https://github.com/postgres/postgres/blob/REL_12_0/src/backend/utils/adt/xml.c#L418
     */
    fn from_binary(ty: &crate::pq::Type, raw: Option<&[u8]>) -> crate::Result<Self> {
        let s = String::from_binary(ty, raw)?;

        Self::from_text(ty, Some(&s))
    }
}

#[cfg(test)]
mod test {
    static XML: &'static str = r#"<?xml version="1.0"?>
<!-- Awesome data incoming -->
<data awesome="true">
  <datum>Science</datum>
  <datum><![CDATA[Literature]]></datum>
  <datum>Math &gt; others</datum>
</data>"#;

    crate::sql_test!(
        xml,
        xmltree::Element,
        [(
            format!("'{}'", super::XML),
            xmltree::Element::parse(super::XML.as_bytes()).unwrap()
        )]
    );
}
