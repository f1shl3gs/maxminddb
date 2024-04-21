use crate::decode::{read_control, read_map, read_str, read_str_array, read_usize};
use crate::Error;

#[derive(Debug, Default)]
pub struct Metadata<'a> {
    pub binary_format_major_version: u16,
    pub binary_format_minor_version: u16,
    pub node_count: usize,
    pub record_size: usize,
    pub ip_version: u16,
    pub database_type: &'a str,
    pub languages: Vec<&'a str>,
    pub build_epoch: u64,
    pub description: Vec<(&'a str, &'a str)>,
}

impl<'a> Metadata<'a> {
    pub(crate) fn from_bytes(buf: &'a [u8]) -> Result<Self, Error> {
        let mut offset = 0;
        let (_data_type, size) = read_control(buf, &mut offset)?;
        let mut metadata = Metadata::default();

        for _ in 0..size {
            match read_str(buf, &mut offset)? {
                "binary_format_major_version" => {
                    metadata.binary_format_major_version = read_usize(buf, &mut offset)? as u16;
                }
                "binary_format_minor_version" => {
                    metadata.binary_format_minor_version = read_usize(buf, &mut offset)? as u16;
                }
                "node_count" => metadata.node_count = read_usize(buf, &mut offset)?,
                "record_size" => {
                    metadata.record_size = read_usize(buf, &mut offset)?;
                }
                "ip_version" => metadata.ip_version = read_usize(buf, &mut offset)? as u16,
                "database_type" => metadata.database_type = read_str(buf, &mut offset)?,
                "languages" => metadata.languages = read_str_array(buf, &mut offset)?,
                "build_epoch" => {
                    metadata.build_epoch = read_usize(buf, &mut offset)? as u64;
                }
                "description" => metadata.description = read_map(buf, &mut offset)?,
                field => return Err(Error::UnknownField(field.into())),
            }
        }

        Ok(metadata)
    }
}

pub(crate) fn find_metadata_start(buf: &[u8]) -> Result<usize, Error> {
    const METADATA_START_MARKER: &[u8] = b"\xab\xcd\xefMaxMind.com";

    let window = METADATA_START_MARKER.len();
    let mut pos = buf.len() - window;

    while pos != 0 {
        pos -= 1;

        if METADATA_START_MARKER == &buf[pos..pos + window] {
            return Ok(pos + window);
        }
    }

    Err(Error::MetadataNotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_bytes() {
        let data = std::fs::read("testdata/GeoLite2-City-Test.mmdb").unwrap();

        let offset = find_metadata_start(&data).unwrap();
        let metadata = Metadata::from_bytes(&data[offset..]).unwrap();
        println!("{:#?}", metadata);

        assert_eq!(metadata.binary_format_major_version, 2);
        assert_eq!(metadata.binary_format_minor_version, 0);
        assert_eq!(metadata.record_size, 28);
        assert_eq!(metadata.ip_version, 6);
        assert_eq!(metadata.database_type, "GeoLite2-City");
        assert_eq!(
            metadata.description[0],
            (
                "en",
                "GeoLite2 City Test Database (fake GeoIP2 data, for example purposes only)"
            )
        )
    }
}
