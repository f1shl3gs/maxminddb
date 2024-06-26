use std::net::IpAddr;
use std::path::Path;

use crate::decode::{
    bytes_to_usize, bytes_to_usize_with_prefix, read_bool, read_control, read_pointer, read_str,
    read_usize, Decoder, DATA_TYPE_MAP, DATA_TYPE_POINTER, DATA_TYPE_SLICE,
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
    pub fn mmap(path: impl AsRef<Path>) -> Result<Reader<memmap2::Mmap>, Error> {
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
        };

        if ip_version == 6 {
            // We are looking up an IPv4 address in an IPv6 tree.
            // Skip over the first 96 nodes.
            let mut node = 0usize;
            for _ in 0..96 {
                if node >= node_count {
                    break;
                }

                node = reader.read_node(node, 0);
            }

            reader.ip_v4_start = node;
        }

        Ok(reader)
    }

    // metadata() is a cold path definitely, so it's ok to decode when
    // we call it.
    pub fn metadata(&'a self) -> Result<Metadata<'a>, Error> {
        let buf = self.data.as_ref();
        let offset = find_metadata_start(buf)?;
        Metadata::from_bytes(&buf[offset..])
    }

    /// Lookup the socket address in the opened MaxMind DB
    pub fn lookup<T: Decoder<'a>>(&'a self, addr: IpAddr) -> Result<T, Error> {
        let pointer = match addr {
            IpAddr::V4(addr) => self.find_address_in_tree(&addr.octets())?,
            IpAddr::V6(addr) => {
                if self.ip_v4_start == 0 {
                    return Err(Error::IPv4Only);
                }

                self.find_address_in_tree(&addr.octets())?
            }
        };
        if pointer == 0 {
            return Err(Error::AddressNotFound);
        }

        let mut offset = pointer - self.node_count - DATA_SECTION_SEPARATOR_SIZE;
        let buf = self.data.as_ref();
        if offset >= buf.len() {
            return Err(Error::CorruptSearchTree);
        }

        // `T` must be a MAP
        let buf = &buf[self.search_tree_size + DATA_SECTION_SEPARATOR_SIZE..];
        let (data_type, size) = read_control(buf, &mut offset)?;
        if data_type != DATA_TYPE_MAP {
            return Err(Error::InvalidDataType(data_type));
        }

        T::decode_with_size(buf, &mut offset, size)
    }

    fn find_address_in_tree(&self, ip: &[u8]) -> Result<usize, Error> {
        let bit_count = ip.len() * 8;
        let mut node: usize = if bit_count == 128 {
            0
        } else {
            self.ip_v4_start
        };

        // node buf
        for i in 0..bit_count {
            if node >= self.node_count {
                break;
            }

            let bit = 1 & (ip[i >> 3] >> (7 - (i % 8)));
            node = self.read_node(node, bit as usize);
        }

        if self.node_count == node {
            Ok(0)
        } else if node > self.node_count {
            Ok(node)
        } else {
            Err(Error::InvalidNode)
        }
    }

    #[inline]
    fn read_node(&self, node: usize, index: usize) -> usize {
        let buf = self.data.as_ref();
        let base = node * self.node_offset_multi;

        match self.record_size {
            28 => {
                let mut middle = buf[base + 3];
                if index != 0 {
                    middle &= 0x0F
                } else {
                    middle = (0xF0 & middle) >> 4
                }

                let offset = base + index * 4;
                bytes_to_usize_with_prefix(middle as usize, &buf[offset..offset + 3])
            }
            24 => {
                let offset = base + index * 3;
                bytes_to_usize(&buf[offset..offset + 3])
            }
            32 => {
                let offset = base + index * 4;
                bytes_to_usize(&buf[offset..offset + 4])
            }
            // record_size is validated at the very beginning
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
    #[inline]
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut city = City::default();

        for _ in 0..size {
            match read_str(buf, offset)? {
                "city" => city.city = Some(models::City::decode(buf, offset)?),
                "continent" => city.continent = Some(models::Continent::decode(buf, offset)?),
                "country" => city.country = Some(models::Country::decode(buf, offset)?),
                "location" => city.location = Some(models::Location::decode(buf, offset)?),
                "postal" => city.postal = Some(models::Postal::decode(buf, offset)?),
                "registered_country" => {
                    city.registered_country = Some(models::Country::decode(buf, offset)?)
                }
                "represented_country" => {
                    city.represented_country =
                        Some(models::RepresentedCountry::decode(buf, offset)?)
                }
                "subdivisions" => {
                    let (data_type, size) = read_control(buf, offset)?;
                    city.subdivisions = match data_type {
                        DATA_TYPE_SLICE => {
                            let mut array = Vec::with_capacity(size);
                            for _ in 0..size {
                                let item = models::Subdivision::decode(buf, offset)?;
                                array.push(item);
                            }

                            Some(array)
                        }
                        DATA_TYPE_POINTER => {
                            let offset = &mut read_pointer(buf, offset, size)?;
                            let (data_type, size) = read_control(buf, offset)?;
                            match data_type {
                                DATA_TYPE_SLICE => {
                                    let mut array = Vec::with_capacity(size);
                                    for _ in 0..size {
                                        let item = models::Subdivision::decode(buf, offset)?;
                                        array.push(item);
                                    }

                                    Some(array)
                                }
                                _ => return Err(Error::InvalidDataType(data_type)),
                            }
                        }
                        _ => return Err(Error::InvalidDataType(data_type)),
                    };
                }
                "traits" => city.traits = Some(models::Traits::decode(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(city)
    }
}

/// GeoIP2 Enterprise record
#[derive(Debug, Default)]
pub struct Enterprise<'a> {
    pub continent: Option<models::Continent<'a>>,
    pub country: Option<models::EnterpriseCountry<'a>>,
    pub subdivisions: Option<Vec<models::EnterpriseSubdivision<'a>>>,
    pub city: Option<models::EnterpriseCity<'a>>,
    pub location: Option<models::Location<'a>>,
    pub postal: Option<models::EnterprisePostal<'a>>,
    pub registered_country: Option<models::EnterpriseCountry<'a>>,
    pub represented_country: Option<models::EnterpriseRepresentedCountry<'a>>,
    pub traits: Option<models::EnterpriseTraits<'a>>,
}

impl<'a> Decoder<'a> for Enterprise<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut enterprise = Enterprise::default();

        for _ in 0..size {
            match read_str(buf, offset)? {
                "city" => enterprise.city = Some(models::EnterpriseCity::decode(buf, offset)?),
                "continent" => enterprise.continent = Some(models::Continent::decode(buf, offset)?),
                "country" => {
                    enterprise.country = Some(models::EnterpriseCountry::decode(buf, offset)?)
                }
                "location" => enterprise.location = Some(models::Location::decode(buf, offset)?),
                "postal" => {
                    enterprise.postal = Some(models::EnterprisePostal::decode(buf, offset)?)
                }
                "registered_country" => {
                    enterprise.registered_country =
                        Some(models::EnterpriseCountry::decode(buf, offset)?)
                }
                "represented_country" => {
                    enterprise.represented_country =
                        Some(models::EnterpriseRepresentedCountry::decode(buf, offset)?)
                }
                "subdivisions" => {
                    let (data_type, size) = read_control(buf, offset)?;

                    enterprise.subdivisions = Some(match data_type {
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
                    });
                }
                "traits" => {
                    enterprise.traits = Some(models::EnterpriseTraits::decode(buf, offset)?)
                }
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(enterprise)
    }
}

/// GeoIP2 Connection-Type record
#[derive(Clone, Debug, Default)]
pub struct ConnectionType<'a> {
    pub connection_type: Option<&'a str>,
}

impl<'a> Decoder<'a> for ConnectionType<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut connection_type = ConnectionType::default();

        for _ in 0..size {
            match read_str(buf, offset)? {
                "connection_type" => connection_type.connection_type = Some(read_str(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(connection_type)
    }
}

/// GeoIP2 Domain record
#[derive(Clone, Debug, Default)]
pub struct Domain<'a> {
    pub domain: Option<&'a str>,
}

impl<'a> Decoder<'a> for Domain<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut domain = Domain::default();

        for _ in 0..size {
            match read_str(buf, offset)? {
                "domain" => domain.domain = Some(read_str(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(domain)
    }
}

/// GeoIP2 ISP record
#[derive(Clone, Debug, Default)]
pub struct Isp<'a> {
    pub autonomous_system_number: Option<u32>,
    pub autonomous_system_organization: Option<&'a str>,
    pub isp: Option<&'a str>,
    pub mobile_country_code: Option<&'a str>,
    pub mobile_network_code: Option<&'a str>,
    pub organization: Option<&'a str>,
}

impl<'a> Decoder<'a> for Isp<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut isp = Isp::default();

        for _ in 0..size {
            match read_str(buf, offset)? {
                "autonomous_system_number" => {
                    isp.autonomous_system_number = Some(read_usize(buf, offset)? as u32)
                }
                "autonomous_system_organization" => {
                    isp.autonomous_system_organization = Some(read_str(buf, offset)?)
                }
                "isp" => isp.isp = Some(read_str(buf, offset)?),
                "mobile_country_code" => isp.mobile_country_code = Some(read_str(buf, offset)?),
                "mobile_network_code" => isp.mobile_network_code = Some(read_str(buf, offset)?),
                "organization" => isp.organization = Some(read_str(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(isp)
    }
}

/// GeoIP2 Asn record
#[derive(Clone, Debug, Default)]
pub struct Asn<'a> {
    pub autonomous_system_number: Option<u32>,
    pub autonomous_system_organization: Option<&'a str>,
}

impl<'a> Decoder<'a> for Asn<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut asn = Asn::default();

        for _ in 0..size {
            match read_str(buf, offset)? {
                "autonomous_system_number" => {
                    asn.autonomous_system_number = Some(read_usize(buf, offset)? as u32)
                }
                "autonomous_system_organization" => {
                    asn.autonomous_system_organization = Some(read_str(buf, offset)?)
                }
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(asn)
    }
}
