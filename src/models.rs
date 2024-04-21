use crate::decode::{read_bool, read_f64, read_map, read_str, read_usize, Decoder};
use crate::Error;

#[derive(Debug)]
pub struct City<'a> {
    pub geoname_id: Option<u32>,
    pub names: Option<Vec<(&'a str, &'a str)>>,
}

impl<'a> Decoder<'a> for City<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut geoname_id = None;
        let mut names = None;

        for _ in 0..size {
            match read_str(buf, offset)? {
                "geoname_id" => geoname_id = Some(read_usize(buf, offset)? as u32),
                "names" => names = Some(read_map(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(Self { geoname_id, names })
    }
}

#[derive(Debug)]
pub struct Continent<'a> {
    pub code: Option<&'a str>,
    pub geoname_id: Option<u32>,
    pub names: Option<Vec<(&'a str, &'a str)>>,
}

impl<'a> Decoder<'a> for Continent<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut code = None;
        let mut geoname_id = None;
        let mut names = None;

        for _ in 0..size {
            match read_str(buf, offset)? {
                "code" => code = Some(read_str(buf, offset)?),
                "geoname_id" => geoname_id = Some(read_usize(buf, offset)? as u32),
                "names" => names = Some(read_map(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(Self {
            code,
            geoname_id,
            names,
        })
    }
}

#[derive(Debug)]
pub struct Country<'a> {
    pub geoname_id: Option<u32>,
    pub is_in_european_union: Option<bool>,
    pub iso_code: Option<&'a str>,
    pub names: Option<Vec<(&'a str, &'a str)>>,
}

impl<'a> Decoder<'a> for Country<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut geoname_id = None;
        let mut is_in_european_union = None;
        let mut iso_code = None;
        let mut names = None;

        for _ in 0..size {
            match read_str(buf, offset)? {
                "geoname_id" => geoname_id = Some(read_usize(buf, offset)? as u32),
                "is_in_european_union" => is_in_european_union = Some(read_bool(buf, offset)?),
                "iso_code" => iso_code = Some(read_str(buf, offset)?),
                "names" => names = Some(read_map(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(Self {
            geoname_id,
            is_in_european_union,
            iso_code,
            names,
        })
    }
}

#[derive(Debug)]
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
        let mut geoname_id = None;
        let mut is_in_european_union = None;
        let mut iso_code = None;
        let mut names = None;
        let mut representation_type = None;

        for _ in 0..size {
            match read_str(buf, offset)? {
                "geoname_id" => geoname_id = Some(read_usize(buf, offset)? as u32),
                "is_in_european_union" => is_in_european_union = Some(read_bool(buf, offset)?),
                "iso_code" => iso_code = Some(read_str(buf, offset)?),
                "names" => names = Some(read_map(buf, offset)?),
                "type" => representation_type = Some(read_str(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(Self {
            geoname_id,
            is_in_european_union,
            iso_code,
            names,
            representation_type,
        })
    }
}

#[derive(Debug)]
pub struct Traits {
    pub is_anonymous_proxy: Option<bool>,
    pub is_anycast: Option<bool>,
    pub is_satellite_provider: Option<bool>,
}

impl<'a> Decoder<'a> for Traits {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut is_anonymous_proxy = None;
        let mut is_anycast = None;
        let mut is_satellite_provider = None;

        for _ in 0..size {
            match read_str(buf, offset)? {
                "is_anonymous_proxy" => is_anonymous_proxy = Some(read_bool(buf, offset)?),
                "is_anycast" => is_anycast = Some(read_bool(buf, offset)?),
                "is_satellite_provider" => is_satellite_provider = Some(read_bool(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(Self {
            is_anonymous_proxy,
            is_anycast,
            is_satellite_provider,
        })
    }
}

#[derive(Debug)]
pub struct Location<'a> {
    pub accuracy_radius: Option<u16>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub metro_code: Option<u16>,
    pub time_zone: Option<&'a str>,
}

impl<'a> Decoder<'a> for Location<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut accuracy_radius = None;
        let mut latitude = None;
        let mut longitude = None;
        let mut metro_code = None;
        let mut time_zone = None;

        for _ in 0..size {
            match read_str(buf, offset)? {
                "accuracy_radius" => accuracy_radius = Some(read_usize(buf, offset)? as u16),
                "latitude" => latitude = Some(read_f64(buf, offset)?),
                "longitude" => longitude = Some(read_f64(buf, offset)?),
                "metro_code" => metro_code = Some(read_usize(buf, offset)? as u16),
                "time_zone" => time_zone = Some(read_str(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(Self {
            accuracy_radius,
            latitude,
            longitude,
            metro_code,
            time_zone,
        })
    }
}

#[derive(Debug)]
pub struct Postal<'a> {
    pub code: Option<&'a str>,
}

impl<'a> Decoder<'a> for Postal<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut code = None;

        for _ in 0..size {
            match read_str(buf, offset)? {
                "code" => code = Some(read_str(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(Self { code })
    }
}

#[derive(Debug)]
pub struct Subdivision<'a> {
    pub geoname_id: Option<u32>,
    pub iso_code: Option<&'a str>,
    pub names: Option<Vec<(&'a str, &'a str)>>,
}

impl<'a> Decoder<'a> for Subdivision<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut geoname_id = None;
        let mut iso_code = None;
        let mut names = None;

        for _ in 0..size {
            match read_str(buf, offset)? {
                "geoname_id" => geoname_id = Some(read_usize(buf, offset)? as u32),
                "iso_code" => iso_code = Some(read_str(buf, offset)?),
                "names" => names = Some(read_map(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(Self {
            geoname_id,
            iso_code,
            names,
        })
    }
}

#[derive(Debug)]
pub struct EnterpriseCountry<'a> {
    pub confidence: Option<u8>,
    pub geoname_id: Option<u32>,
    pub is_in_european_union: Option<bool>,
    pub iso_code: Option<&'a str>,
    pub names: Option<Vec<(&'a str, &'a str)>>,
}

impl<'a> Decoder<'a> for EnterpriseCountry<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut confidence = None;
        let mut geoname_id = None;
        let mut is_in_european_union = None;
        let mut iso_code = None;
        let mut names = None;

        for _ in 0..size {
            match read_str(buf, offset)? {
                "confidence" => confidence = Some(read_usize(buf, offset)? as u8),
                "geoname_id" => geoname_id = Some(read_usize(buf, offset)? as u32),
                "is_in_european_union" => is_in_european_union = Some(read_bool(buf, offset)?),
                "iso_code" => iso_code = Some(read_str(buf, offset)?),
                "names" => names = Some(read_map(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(Self {
            confidence,
            geoname_id,
            is_in_european_union,
            iso_code,
            names,
        })
    }
}

#[derive(Debug)]
pub struct EnterpriseCity<'a> {
    pub confidence: Option<u8>,
    pub geoname_id: Option<u32>,
    pub names: Option<Vec<(&'a str, &'a str)>>,
}

impl<'a> Decoder<'a> for EnterpriseCity<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut confidence = None;
        let mut geoname_id = None;
        let mut names = None;

        for _ in 0..size {
            match read_str(buf, offset)? {
                "confidence" => confidence = Some(read_usize(buf, offset)? as u8),
                "geoname_id" => geoname_id = Some(read_usize(buf, offset)? as u32),
                "names" => names = Some(read_map(buf, offset)?),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(Self {
            confidence,
            geoname_id,
            names,
        })
    }
}

#[derive(Debug)]
pub struct EnterprisePostal<'a> {
    pub code: Option<&'a str>,
    pub confidence: Option<u8>,
}

impl<'a> Decoder<'a> for EnterprisePostal<'a> {
    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error> {
        let mut code = None;
        let mut confidence = None;

        for _ in 0..size {
            match read_str(buf, offset)? {
                "code" => code = Some(read_str(buf, offset)?),
                "confidence" => confidence = Some(read_usize(buf, offset)? as u8),
                field => return Err(Error::UnknownField(field.to_string())),
            }
        }

        Ok(Self { code, confidence })
    }
}

#[derive(Debug)]
pub struct EnterpriseSubdivision<'a> {
    pub confidence: Option<u8>,
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
                "confidence" => confidence = Some(read_usize(buf, offset)? as u8),
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

#[derive(Debug)]
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
            is_tor_exit_node,
            mobile_country_code,
            mobile_network_code,
            organization,
            user_type,
        })
    }
}
