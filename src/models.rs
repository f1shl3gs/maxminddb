use crate::decode::{read_bool, read_f64, read_map, read_str, read_usize, Decoder};
use crate::Error;

#[derive(Debug, Default)]
pub struct City<'a> {
    pub geoname_id: Option<u32>,
    pub names: Option<Vec<(&'a str, &'a str)>>,
}

impl<'a> Decoder<'a> for City<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut city = City::default();

        for _ in 0..size {
            match read_str(buf, offset)? {
                "geoname_id" => city.geoname_id = Some(read_usize(buf, offset)? as u32),
                "names" => city.names = Some(read_map(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(city)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Continent<'a> {
    pub geoname_id: Option<u32>,
    pub code: Option<&'a str>,
    pub names: Option<Vec<(&'a str, &'a str)>>,
}

impl<'a> Decoder<'a> for Continent<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut continent = Continent::default();

        for _ in 0..size {
            match read_str(buf, offset)? {
                "geoname_id" => continent.geoname_id = Some(read_usize(buf, offset)? as u32),
                "code" => continent.code = Some(read_str(buf, offset)?),
                "names" => continent.names = Some(read_map(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(continent)
    }
}

#[derive(Debug, Default)]
pub struct Country<'a> {
    pub geoname_id: Option<u32>,
    pub is_in_european_union: Option<bool>,
    pub iso_code: Option<&'a str>,
    pub names: Option<Vec<(&'a str, &'a str)>>,
}

impl<'a> Decoder<'a> for Country<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut country = Country::default();

        for _ in 0..size {
            match read_str(buf, offset)? {
                "geoname_id" => country.geoname_id = Some(read_usize(buf, offset)? as u32),
                "is_in_european_union" => {
                    country.is_in_european_union = Some(read_bool(buf, offset)?)
                }
                "iso_code" => country.iso_code = Some(read_str(buf, offset)?),
                "names" => country.names = Some(read_map(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(country)
    }
}

#[derive(Clone, Debug, Default)]
pub struct RepresentedCountry<'a> {
    pub geoname_id: Option<u32>,
    pub is_in_european_union: Option<bool>,
    pub iso_code: Option<&'a str>,
    pub names: Option<Vec<(&'a str, &'a str)>>,
    // type actually
    pub representation_type: Option<&'a str>,
}

impl<'a> Decoder<'a> for RepresentedCountry<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut represented_country = RepresentedCountry::default();

        for _ in 0..size {
            match read_str(buf, offset)? {
                "geoname_id" => {
                    represented_country.geoname_id = Some(read_usize(buf, offset)? as u32)
                }
                "is_in_european_union" => {
                    represented_country.is_in_european_union = Some(read_bool(buf, offset)?)
                }
                "iso_code" => represented_country.iso_code = Some(read_str(buf, offset)?),
                "names" => represented_country.names = Some(read_map(buf, offset)?),
                "type" => represented_country.representation_type = Some(read_str(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(represented_country)
    }
}

#[derive(Debug, Default)]
pub struct Traits {
    pub is_anonymous_proxy: Option<bool>,
    pub is_anycast: Option<bool>,
    pub is_satellite_provider: Option<bool>,
}

impl<'a> Decoder<'a> for Traits {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut traits = Traits::default();

        for _ in 0..size {
            match read_str(buf, offset)? {
                "is_anonymous_proxy" => traits.is_anonymous_proxy = Some(read_bool(buf, offset)?),
                "is_anycast" => traits.is_anycast = Some(read_bool(buf, offset)?),
                "is_satellite_provider" => {
                    traits.is_satellite_provider = Some(read_bool(buf, offset)?)
                }
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(traits)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Location<'a> {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub accuracy_radius: Option<u16>,
    pub time_zone: Option<&'a str>,
    pub metro_code: Option<u16>,
}

impl<'a> Decoder<'a> for Location<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut location = Location::default();

        for _ in 0..size {
            match read_str(buf, offset)? {
                "latitude" => location.latitude = Some(read_f64(buf, offset)?),
                "longitude" => location.longitude = Some(read_f64(buf, offset)?),
                "accuracy_radius" => {
                    location.accuracy_radius = Some(read_usize(buf, offset)? as u16)
                }
                "time_zone" => location.time_zone = Some(read_str(buf, offset)?),
                "metro_code" => location.metro_code = Some(read_usize(buf, offset)? as u16),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(location)
    }
}

#[derive(Debug, Default)]
pub struct Postal<'a> {
    pub code: Option<&'a str>,
}

impl<'a> Decoder<'a> for Postal<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut postal = Postal::default();

        for _ in 0..size {
            match read_str(buf, offset)? {
                "code" => postal.code = Some(read_str(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(postal)
    }
}

#[derive(Debug, Default)]
pub struct Subdivision<'a> {
    pub geoname_id: Option<u32>,
    pub iso_code: Option<&'a str>,
    pub names: Option<Vec<(&'a str, &'a str)>>,
}

impl<'a> Decoder<'a> for Subdivision<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut subdivision = Subdivision::default();

        for _ in 0..size {
            match read_str(buf, offset)? {
                "geoname_id" => subdivision.geoname_id = Some(read_usize(buf, offset)? as u32),
                "iso_code" => subdivision.iso_code = Some(read_str(buf, offset)?),
                "names" => subdivision.names = Some(read_map(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(subdivision)
    }
}

#[derive(Clone, Debug, Default)]
pub struct EnterpriseCountry<'a> {
    pub geoname_id: Option<u32>,
    pub iso_code: Option<&'a str>,
    pub names: Option<Vec<(&'a str, &'a str)>>,
    pub is_in_european_union: Option<bool>,
    pub confidence: Option<u16>,
}

impl<'a> Decoder<'a> for EnterpriseCountry<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut enterprise_country = EnterpriseCountry::default();

        for _ in 0..size {
            match read_str(buf, offset)? {
                "confidence" => {
                    enterprise_country.confidence = Some(read_usize(buf, offset)? as u16)
                }
                "geoname_id" => {
                    enterprise_country.geoname_id = Some(read_usize(buf, offset)? as u32)
                }
                "iso_code" => enterprise_country.iso_code = Some(read_str(buf, offset)?),
                "names" => enterprise_country.names = Some(read_map(buf, offset)?),
                "is_in_european_union" => {
                    enterprise_country.is_in_european_union = Some(read_bool(buf, offset)?)
                }
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(enterprise_country)
    }
}

#[derive(Clone, Debug, Default)]
pub struct EnterpriseRepresentedCountry<'a> {
    pub confidence: Option<u16>,
    pub geoname_id: Option<u32>,
    pub iso_code: Option<&'a str>,
    pub names: Option<Vec<(&'a str, &'a str)>>,
    pub is_in_european_union: Option<bool>,
    pub country_type: Option<&'a str>,
}

impl<'a> Decoder<'a> for EnterpriseRepresentedCountry<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut country = EnterpriseRepresentedCountry::default();

        for _ in 0..size {
            match read_str(buf, offset)? {
                "confidence" => country.confidence = Some(read_usize(buf, offset)? as u16),
                "geoname_id" => country.geoname_id = Some(read_usize(buf, offset)? as u32),
                "iso_code" => country.iso_code = Some(read_str(buf, offset)?),
                "names" => country.names = Some(read_map(buf, offset)?),
                "is_in_european_union" => {
                    country.is_in_european_union = Some(read_bool(buf, offset)?)
                }
                "country_type" => country.country_type = Some(read_str(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(country)
    }
}

#[derive(Clone, Debug, Default)]
pub struct EnterpriseCity<'a> {
    pub confidence: Option<u16>,
    pub geoname_id: Option<u32>,
    pub names: Option<Vec<(&'a str, &'a str)>>,
}

impl<'a> Decoder<'a> for EnterpriseCity<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut city = EnterpriseCity::default();

        for _ in 0..size {
            match read_str(buf, offset)? {
                "confidence" => city.confidence = Some(read_usize(buf, offset)? as u16),
                "geoname_id" => city.geoname_id = Some(read_usize(buf, offset)? as u32),
                "names" => city.names = Some(read_map(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(city)
    }
}

#[derive(Clone, Debug, Default)]
pub struct EnterprisePostal<'a> {
    pub confidence: Option<u16>,
    pub code: Option<&'a str>,
}

impl<'a> Decoder<'a> for EnterprisePostal<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut postal = EnterprisePostal::default();

        for _ in 0..size {
            match read_str(buf, offset)? {
                "confidence" => postal.confidence = Some(read_usize(buf, offset)? as u16),
                "code" => postal.code = Some(read_str(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(postal)
    }
}

#[derive(Clone, Debug)]
pub struct EnterpriseSubdivision<'a> {
    pub confidence: Option<u16>,
    pub geoname_id: Option<u32>,
    pub iso_code: Option<&'a str>,
    pub names: Option<Vec<(&'a str, &'a str)>>,
}

impl<'a> Decoder<'a> for EnterpriseSubdivision<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut confidence = None;
        let mut geoname_id = None;
        let mut iso_code = None;
        let mut names = None;

        for _ in 0..size {
            match read_str(buf, offset)? {
                "confidence" => confidence = Some(read_usize(buf, offset)? as u16),
                "geoname_id" => geoname_id = Some(read_usize(buf, offset)? as u32),
                "iso_code" => iso_code = Some(read_str(buf, offset)?),
                "names" => names = Some(read_map(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(Self {
            confidence,
            geoname_id,
            iso_code,
            names,
        })
    }
}

#[derive(Clone, Debug)]
pub struct EnterpriseTraits<'a> {
    pub autonomous_system_number: Option<u32>,
    pub autonomous_system_organization: Option<&'a str>,
    pub connection_type: Option<&'a str>,
    pub domain: Option<&'a str>,
    pub is_anonymous: Option<bool>,
    pub is_anonymous_proxy: Option<bool>,
    pub is_anonymous_vpn: Option<bool>,
    pub is_anycast: Option<bool>,
    pub is_hosting_provider: Option<bool>,
    pub isp: Option<&'a str>,
    pub is_public_proxy: Option<bool>,
    pub is_residential_proxy: Option<bool>,
    pub is_satellite_provider: Option<bool>,
    pub is_legitimate_proxy: Option<bool>,
    pub static_ip_score: Option<f64>,
    pub is_tor_exit_node: Option<bool>,
    pub mobile_country_code: Option<&'a str>,
    pub mobile_network_code: Option<&'a str>,
    pub organization: Option<&'a str>,
    pub user_type: Option<&'a str>,
}

impl<'a> Decoder<'a> for EnterpriseTraits<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut autonomous_system_number = None;
        let mut autonomous_system_organization = None;
        let mut connection_type = None;
        let mut domain = None;
        let mut is_anonymous = None;
        let mut is_anonymous_proxy = None;
        let mut is_anonymous_vpn = None;
        let mut is_anycast = None;
        let mut is_hosting_provider = None;
        let mut isp = None;
        let mut is_public_proxy = None;
        let mut is_residential_proxy = None;
        let mut is_satellite_provider = None;
        let mut is_legitimate_proxy = None;
        let mut static_ip_score = None;
        let mut is_tor_exit_node = None;
        let mut mobile_country_code = None;
        let mut mobile_network_code = None;
        let mut organization = None;
        let mut user_type = None;

        for _ in 0..size {
            match read_str(buf, offset)? {
                "autonomous_system_number" => {
                    autonomous_system_number = Some(read_usize(buf, offset)? as u32)
                }
                "autonomous_system_organization" => {
                    autonomous_system_organization = Some(read_str(buf, offset)?)
                }
                "connection_type" => connection_type = Some(read_str(buf, offset)?),
                "domain" => domain = Some(read_str(buf, offset)?),
                "is_anonymous" => is_anonymous = Some(read_bool(buf, offset)?),
                "is_anonymous_proxy" => is_anonymous_proxy = Some(read_bool(buf, offset)?),
                "is_anonymous_vpn" => is_anonymous_vpn = Some(read_bool(buf, offset)?),
                "is_anycast" => is_anycast = Some(read_bool(buf, offset)?),
                "is_hosting_provider" => is_hosting_provider = Some(read_bool(buf, offset)?),
                "isp" => isp = Some(read_str(buf, offset)?),
                "is_public_proxy" => is_public_proxy = Some(read_bool(buf, offset)?),
                "is_residential_proxy" => is_residential_proxy = Some(read_bool(buf, offset)?),
                "is_satellite_provider" => is_satellite_provider = Some(read_bool(buf, offset)?),
                "is_legitimate_proxy" => is_legitimate_proxy = Some(read_bool(buf, offset)?),
                "static_ip_score" => static_ip_score = Some(read_f64(buf, offset)?),
                "is_tor_exit_node" => is_tor_exit_node = Some(read_bool(buf, offset)?),
                "mobile_country_code" => mobile_country_code = Some(read_str(buf, offset)?),
                "mobile_network_code" => mobile_network_code = Some(read_str(buf, offset)?),
                "organization" => organization = Some(read_str(buf, offset)?),
                "user_type" => user_type = Some(read_str(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(Self {
            autonomous_system_number,
            autonomous_system_organization,
            connection_type,
            domain,
            is_anonymous,
            is_anonymous_proxy,
            is_anonymous_vpn,
            is_anycast,
            is_hosting_provider,
            isp,
            is_public_proxy,
            is_residential_proxy,
            is_satellite_provider,
            is_legitimate_proxy,
            static_ip_score,
            is_tor_exit_node,
            mobile_country_code,
            mobile_network_code,
            organization,
            user_type,
        })
    }
}
