use std::net::IpAddr;
use std::path::Path;

use crate::decode::{
    read_bool, read_control, read_pointer, read_str, read_usize, Decoder, DATA_TYPE_MAP,
    DATA_TYPE_POINTER, DATA_TYPE_SLICE,
};
use crate::metadata::{find_metadata_start, Metadata};
use crate::{models, Error};

const DATA_SECTION_SEPARATOR_SIZE: usize = 16;

/// A reader for the MaxMind DB format. The lifetime 'data' is tied to the lifetime
/// of the underlying buffer holding the content of the database file.
pub struct Reader<S: AsRef<[u8]>> {
    data: S,

    search_tree_size: usize,
    record_size: usize,
    node_count: usize,
    node_offset_multi: usize,
    ip_v4_start: usize,
    ip_v4_start_bit_depth: usize,
}

impl Reader<Vec<u8>> {
    /// Open a maxMind DB file by loading it into memory.
    pub fn open_file(path: impl AsRef<Path>) -> Result<Reader<Vec<u8>>, Error> {
        let data = std::fs::read(path)?;
        Reader::from_bytes(data)
    }
}

#[cfg(feature = "mmap")]
impl Reader<memmap2::Mmap> {
    /// Open a MaxMind DB file by mmaping it.
    pub fn open_mmap(path: impl AsRef<Path>) -> Result<Reader<memmap2::Mmap>, Error> {
        let file = std::fs::File::open(path)?;
        let mmap = unsafe { memmap2::MmapOptions::new().map(&file) }?;
        Reader::from_bytes(mmap)
    }
}

impl<'a, S: AsRef<[u8]>> Reader<S> {
    /// Open a MaxMind DB database from anything that implements AsRef<[u8]>
    pub fn from_bytes(buf: S) -> Result<Self, Error> {
        let metadata_start = find_metadata_start(buf.as_ref())?;
        let metadata = Metadata::from_bytes(&buf.as_ref()[metadata_start..])?;

        // validate metadata
        if metadata.record_size != 24 && metadata.record_size != 28 && metadata.record_size != 32 {
            return Err(Error::InvalidRecordSize(metadata.record_size));
        }

        let record_size = metadata.record_size;
        let node_count = metadata.node_count;
        let ip_version = metadata.ip_version;
        let node_offset_multi = record_size / 4;
        let search_tree_size = node_count * node_offset_multi;
        let data_section_start = search_tree_size + DATA_SECTION_SEPARATOR_SIZE;
        if data_section_start > metadata_start {
            return Err(Error::InvalidSearchTreeSize);
        }

        let mut reader = Reader {
            data: buf,
            search_tree_size,
            record_size,
            node_count,
            node_offset_multi,
            ip_v4_start: 0,
            ip_v4_start_bit_depth: 0,
        };

        if ip_version == 6 {
            let mut node = 0usize;
            let mut i = 0usize;
            while i < 96 && node < node_count {
                i += 1;
                let buf = &reader.data.as_ref()[..search_tree_size];
                node = reader.read_left(buf, node * node_offset_multi);
            }

            reader.ip_v4_start = node;
            reader.ip_v4_start_bit_depth = i;
        }

        Ok(reader)
    }

    // metadata() is a code path definitely, so it's ok to decode when
    pub fn metadata(&'a self) -> Result<Metadata<'a>, Error> {
        let buf = self.data.as_ref();
        let offset = find_metadata_start(buf)?;
        Metadata::from_bytes(&buf[offset..])
    }

    /// Lookup the socket address in the opened MaxMind DB
    pub fn lookup<T: Decoder<'a>>(&'a self, addr: IpAddr) -> Result<T, Error> {
        let pointer = match addr {
            IpAddr::V4(addr) => self.find_address_in_tree(addr.octets().as_ref())?,
            IpAddr::V6(addr) => self.find_address_in_tree(addr.octets().as_ref())?,
        };
        if pointer == 0 {
            return Err(Error::AddressNotFound);
        }

        let mut offset = pointer - self.node_count - DATA_SECTION_SEPARATOR_SIZE;
        if offset >= self.data.as_ref().len() {
            return Err(Error::CorruptSearchTree);
        }

        T::decode(
            &self.data.as_ref()[self.search_tree_size + DATA_SECTION_SEPARATOR_SIZE..],
            &mut offset,
        )
    }

    fn find_address_in_tree(&self, ip: &[u8]) -> Result<usize, Error> {
        let bit_count = ip.len() * 8;
        let mut node: usize = if bit_count == 128 {
            0
        } else {
            self.ip_v4_start
        };

        // node buf
        let buf = &self.data.as_ref()[..self.search_tree_size];
        for i in 0..bit_count {
            if node >= self.node_count {
                break;
            }

            let bit = 1 & (ip[i >> 3] >> (7 - (i % 8)));
            let offset = node * self.node_offset_multi;
            node = if bit == 0 {
                self.read_left(buf, offset)
            } else {
                self.read_right(buf, offset)
            }
        }

        match self.node_count {
            n if n == node => Ok(0),
            n if node > n => Ok(node),
            _ => Err(Error::InvalidNode),
        }
    }

    fn read_left(&self, buf: &[u8], nodes: usize) -> usize {
        match self.record_size {
            28 => {
                (((buf[nodes + 3] as usize) & 0xF0) << 20)
                    | ((buf[nodes] as usize) << 16)
                    | ((buf[nodes + 1] as usize) << 8)
                    | (buf[nodes + 2] as usize)
            }
            24 => {
                ((buf[nodes] as usize) << 16)
                    | ((buf[nodes + 1] as usize) << 8)
                    | (buf[nodes + 2] as usize)
            }
            32 => {
                ((buf[nodes] as usize) << 24)
                    | ((buf[nodes + 1] as usize) << 16)
                    | ((buf[nodes + 2] as usize) << 8)
                    | (buf[nodes + 3] as usize)
            }
            _ => panic!(),
        }
    }

    fn read_right(&self, buf: &[u8], nodes: usize) -> usize {
        match self.record_size {
            28 => {
                (((buf[nodes + 3] as usize) & 0x0F) << 24)
                    | ((buf[nodes + 4] as usize) << 16)
                    | ((buf[nodes + 5] as usize) << 8)
                    | (buf[nodes + 6] as usize)
            }
            24 => {
                ((buf[nodes + 3] as usize) << 16)
                    | ((buf[nodes + 4] as usize) << 8)
                    | (buf[nodes + 5] as usize)
            }
            32 => {
                ((buf[nodes + 4] as usize) << 24)
                    | ((buf[nodes + 5] as usize) << 16)
                    | ((buf[nodes + 6] as usize) << 8)
                    | (buf[nodes + 7] as usize)
            }
            _ => panic!(),
        }
    }
}

/// GeoIP2 Anonymous Ip record
#[derive(Debug)]
pub struct AnonymousIp {
    pub is_anonymous: Option<bool>,
    pub is_anonymous_vpn: Option<bool>,
    pub is_hosting_provider: Option<bool>,
    pub is_public_proxy: Option<bool>,
    pub is_residential_proxy: Option<bool>,
    pub is_tor_exit_node: Option<bool>,
}

impl<'a> Decoder<'a> for AnonymousIp {
    fn decode(buf: &'a [u8], offset: &mut usize) -> Result<Self, Error> {
        let (data_type, size) = read_control(buf, offset)?;
        if data_type != DATA_TYPE_MAP {
            return Err(Error::InvalidDataType(data_type));
        }

        Self::decode_with_size(buf, offset, size)
    }

    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut is_anonymous = None;
        let mut is_anonymous_vpn = None;
        let mut is_hosting_provider = None;
        let mut is_public_proxy = None;
        let mut is_residential_proxy = None;
        let mut is_tor_exit_node = None;

        for _ in 0..size {
            match read_str(buf, offset)? {
                "is_anonymous" => is_anonymous = Some(read_bool(buf, offset)?),
                "is_anonymous_vpn" => is_anonymous_vpn = Some(read_bool(buf, offset)?),
                "is_hosting_provider" => is_hosting_provider = Some(read_bool(buf, offset)?),
                "is_public_proxy" => is_public_proxy = Some(read_bool(buf, offset)?),
                "is_residential_proxy" => is_residential_proxy = Some(read_bool(buf, offset)?),
                "is_tor_exit_node" => is_tor_exit_node = Some(read_bool(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(Self {
            is_anonymous,
            is_anonymous_vpn,
            is_hosting_provider,
            is_public_proxy,
            is_residential_proxy,
            is_tor_exit_node,
        })
    }
}

/// GeoIP2 Country record
#[derive(Debug)]
pub struct Country<'a> {
    pub continent: Option<models::Continent<'a>>,
    pub country: Option<models::Country<'a>>,
    pub registered_country: Option<models::Country<'a>>,
    pub represented_country: Option<models::RepresentedCountry<'a>>,
    pub traits: Option<models::Traits>,
}

impl<'a> Decoder<'a> for Country<'a> {
    fn decode(buf: &'a [u8], offset: &mut usize) -> Result<Self, Error> {
        let (data_type, size) = read_control(buf, offset)?;
        if data_type != DATA_TYPE_MAP {
            return Err(Error::InvalidDataType(data_type));
        }

        Self::decode_with_size(buf, offset, size)
    }

    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut continent = None;
        let mut country = None;
        let mut registered_country = None;
        let mut represented_country = None;
        let mut traits = None;

        for _ in 0..size {
            match read_str(buf, offset)? {
                "continent" => {
                    continent = Some(models::Continent::decode(buf, offset)?);
                }
                "country" => {
                    country = Some(models::Country::decode(buf, offset)?);
                }
                "registered_country" => {
                    registered_country = Some(models::Country::decode(buf, offset)?);
                }
                "represented_country" => {
                    represented_country = Some(models::RepresentedCountry::decode(buf, offset)?)
                }
                "traits" => traits = Some(models::Traits::decode(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(Self {
            continent,
            country,
            registered_country,
            represented_country,
            traits,
        })
    }
}

#[derive(Debug, Default)]
pub struct City<'a> {
    pub city: Option<models::City<'a>>,
    pub continent: Option<models::Continent<'a>>,
    pub country: Option<models::Country<'a>>,
    pub location: Option<models::Location<'a>>,
    pub postal: Option<models::Postal<'a>>,
    pub registered_country: Option<models::Country<'a>>,
    pub represented_country: Option<models::RepresentedCountry<'a>>,
    pub subdivisions: Option<Vec<models::Subdivision<'a>>>,
    pub traits: Option<models::Traits>,
}

impl<'a> Decoder<'a> for City<'a> {
    fn decode(buf: &'a [u8], offset: &mut usize) -> Result<Self, Error> {
        let (data_type, size) = read_control(buf, offset)?;
        if data_type != DATA_TYPE_MAP {
            return Err(Error::InvalidDataType(data_type));
        }

        Self::decode_with_size(buf, offset, size)
    }

    #[inline]
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut city = None;
        let mut continent = None;
        let mut country = None;
        let mut location = None;
        let mut postal = None;
        let mut registered_country = None;
        let mut represented_country = None;
        let mut subdivisions = None;
        let mut traits = None;

        for _ in 0..size {
            match read_str(buf, offset)? {
                "city" => city = Some(models::City::decode(buf, offset)?),
                "continent" => continent = Some(models::Continent::decode(buf, offset)?),
                "country" => country = Some(models::Country::decode(buf, offset)?),
                "location" => location = Some(models::Location::decode(buf, offset)?),
                "postal" => postal = Some(models::Postal::decode(buf, offset)?),
                "registered_country" => {
                    registered_country = Some(models::Country::decode(buf, offset)?)
                }
                "represented_country" => {
                    represented_country = Some(models::RepresentedCountry::decode(buf, offset)?)
                }
                "subdivisions" => {
                    let (data_type, size) = read_control(buf, offset)?;

                    let array = if data_type == DATA_TYPE_SLICE {
                        let mut array = Vec::with_capacity(size);

                        for _ in 0..size {
                            let item = models::Subdivision::decode(buf, offset)?;
                            array.push(item);
                        }

                        array
                    } else if data_type == DATA_TYPE_POINTER {
                        let offset = &mut read_pointer(buf, offset, size)?;
                        let (data_type, size) = read_control(buf, offset)?;
                        match data_type {
                            DATA_TYPE_SLICE => {
                                let mut array = Vec::with_capacity(size);
                                for _ in 0..size {
                                    let item = models::Subdivision::decode(buf, offset)?;
                                    array.push(item);
                                }

                                array
                            }
                            _ => return Err(Error::InvalidDataType(data_type)),
                        }
                    } else {
                        return Err(Error::InvalidDataType(data_type));
                    };

                    subdivisions = Some(array);
                }
                "traits" => traits = Some(models::Traits::decode(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(Self {
            city,
            continent,
            country,
            location,
            postal,
            registered_country,
            represented_country,
            subdivisions,
            traits,
        })
    }
}

/// GeoIP2 Enterprise record
#[derive(Debug)]
pub struct Enterprise<'a> {
    pub city: Option<models::EnterpriseCity<'a>>,
    pub continent: Option<models::Continent<'a>>,
    pub country: Option<models::EnterpriseCountry<'a>>,
    pub location: Option<models::Location<'a>>,
    pub postal: Option<models::EnterprisePostal<'a>>,
    pub registered_country: Option<models::EnterpriseCountry<'a>>,
    pub represented_country: Option<models::RepresentedCountry<'a>>,
    pub subdivisions: Option<Vec<models::EnterpriseSubdivision<'a>>>,
    pub traits: Option<models::EnterpriseTraits<'a>>,
}

impl<'a> Decoder<'a> for Enterprise<'a> {
    fn decode(buf: &'a [u8], offset: &mut usize) -> Result<Self, Error> {
        let (data_type, size) = read_control(buf, offset)?;
        if data_type != DATA_TYPE_MAP {
            return Err(Error::InvalidDataType(data_type));
        }

        Self::decode_with_size(buf, offset, size)
    }

    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut city = None;
        let mut continent = None;
        let mut country = None;
        let mut location = None;
        let mut postal = None;
        let mut registered_country = None;
        let mut represented_country = None;
        let mut subdivisions = None;
        let mut traits = None;

        for _ in 0..size {
            match read_str(buf, offset)? {
                "city" => city = Some(models::EnterpriseCity::decode(buf, offset)?),
                "continent" => continent = Some(models::Continent::decode(buf, offset)?),
                "country" => country = Some(models::EnterpriseCountry::decode(buf, offset)?),
                "location" => location = Some(models::Location::decode(buf, offset)?),
                "postal" => postal = Some(models::EnterprisePostal::decode(buf, offset)?),
                "registered_country" => {
                    registered_country = Some(models::EnterpriseCountry::decode(buf, offset)?)
                }
                "represented_country" => {
                    represented_country = Some(models::RepresentedCountry::decode(buf, offset)?)
                }
                "subdivisions" => {
                    let (data_type, size) = read_control(buf, offset)?;
                    let array = match data_type {
                        DATA_TYPE_SLICE => {
                            let mut array = Vec::with_capacity(size);

                            for _ in 0..size {
                                let item = models::EnterpriseSubdivision::decode(buf, offset)?;
                                array.push(item);
                            }

                            array
                        }
                        DATA_TYPE_POINTER => {
                            let offset = &mut read_pointer(buf, offset, size)?;
                            let (data_type, size) = read_control(buf, offset)?;
                            match data_type {
                                DATA_TYPE_SLICE => {
                                    let mut array = Vec::with_capacity(size);
                                    for _ in 0..size {
                                        let item =
                                            models::EnterpriseSubdivision::decode(buf, offset)?;
                                        array.push(item);
                                    }

                                    array
                                }
                                _ => return Err(Error::InvalidDataType(data_type)),
                            }
                        }
                        _ => return Err(Error::InvalidDataType(data_type)),
                    };

                    subdivisions = Some(array);
                }
                "traits" => traits = Some(models::EnterpriseTraits::decode(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(Self {
            city,
            continent,
            country,
            location,
            postal,
            registered_country,
            represented_country,
            subdivisions,
            traits,
        })
    }
}

/// GeoIP2 Connection-Type record
#[derive(Clone, Debug)]
pub struct ConnectionType<'a> {
    pub connection_type: Option<&'a str>,
}

impl<'a> Decoder<'a> for ConnectionType<'a> {
    fn decode(buf: &'a [u8], offset: &mut usize) -> Result<Self, Error> {
        let (data_type, size) = read_control(buf, offset)?;
        if data_type != DATA_TYPE_MAP {
            return Err(Error::InvalidDataType(data_type));
        }

        Self::decode_with_size(buf, offset, size)
    }

    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut connection_type = None;

        for _ in 0..size {
            match read_str(buf, offset)? {
                "connection_type" => connection_type = Some(read_str(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(Self { connection_type })
    }
}

/// GeoIP2 Domain record
#[derive(Debug)]
pub struct Domain<'a> {
    pub domain: Option<&'a str>,
}

impl<'a> Decoder<'a> for Domain<'a> {
    fn decode(buf: &'a [u8], offset: &mut usize) -> Result<Self, Error> {
        let (data_type, size) = read_control(buf, offset)?;
        if data_type != DATA_TYPE_MAP {
            return Err(Error::InvalidDataType(data_type));
        }

        Self::decode_with_size(buf, offset, size)
    }

    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut domain = None;

        for _ in 0..size {
            match read_str(buf, offset)? {
                "domain" => domain = Some(read_str(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(Self { domain })
    }
}

/// GeoIP2 ISP record
#[derive(Clone, Debug)]
pub struct Isp<'a> {
    pub autonomous_system_number: Option<u32>,
    pub autonomous_system_organization: Option<&'a str>,
    pub isp: Option<&'a str>,
    pub mobile_country_code: Option<&'a str>,
    pub mobile_network_code: Option<&'a str>,
    pub organization: Option<&'a str>,
}

impl<'a> Decoder<'a> for Isp<'a> {
    fn decode(buf: &'a [u8], offset: &mut usize) -> Result<Self, Error> {
        let (data_type, size) = read_control(buf, offset)?;
        if data_type != DATA_TYPE_MAP {
            return Err(Error::InvalidDataType(data_type));
        }

        Self::decode_with_size(buf, offset, size)
    }

    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut autonomous_system_number = None;
        let mut autonomous_system_organization = None;
        let mut isp = None;
        let mut mobile_country_code = None;
        let mut mobile_network_code = None;
        let mut organization = None;

        for _ in 0..size {
            match read_str(buf, offset)? {
                "autonomous_system_number" => {
                    autonomous_system_number = Some(read_usize(buf, offset)? as u32)
                }
                "autonomous_system_organization" => {
                    autonomous_system_organization = Some(read_str(buf, offset)?)
                }
                "isp" => isp = Some(read_str(buf, offset)?),
                "mobile_country_code" => mobile_country_code = Some(read_str(buf, offset)?),
                "mobile_network_code" => mobile_network_code = Some(read_str(buf, offset)?),
                "organization" => organization = Some(read_str(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(Self {
            autonomous_system_number,
            autonomous_system_organization,
            isp,
            mobile_country_code,
            mobile_network_code,
            organization,
        })
    }
}

/// GeoIP2 Asn record
#[derive(Clone, Debug)]
pub struct Asn<'a> {
    pub autonomous_system_number: Option<u32>,
    pub autonomous_system_organization: Option<&'a str>,
}

impl<'a> Decoder<'a> for Asn<'a> {
    fn decode(buf: &'a [u8], offset: &mut usize) -> Result<Self, Error> {
        let (data_type, size) = read_control(buf, offset)?;
        if data_type != DATA_TYPE_MAP {
            return Err(Error::InvalidDataType(data_type));
        }

        Self::decode_with_size(buf, offset, size)
    }

    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut autonomous_system_number = None;
        let mut autonomous_system_organization = None;

        for _ in 0..size {
            match read_str(buf, offset)? {
                "autonomous_system_number" => {
                    autonomous_system_number = Some(read_usize(buf, offset)? as u32)
                }
                "autonomous_system_organization" => {
                    autonomous_system_organization = Some(read_str(buf, offset)?)
                }
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(Self {
            autonomous_system_number,
            autonomous_system_organization,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::str::FromStr;

    #[test]
    fn lookup() {
        let data = std::fs::read("testdata/GeoIP2-City-Test.mmdb").unwrap();
        let reader = Reader::from_bytes(data).unwrap();
        let ip = IpAddr::from_str("81.2.69.142").unwrap();
        let city = reader.lookup::<City>(ip).unwrap();
        println!("{:#?}", city);
    }
}
