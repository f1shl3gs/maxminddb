use crate::Error;

pub(crate) const DATA_TYPE_EXTENDED: u8 = 0;
pub(crate) const DATA_TYPE_POINTER: u8 = 1;
pub(crate) const DATA_TYPE_STRING: u8 = 2;
pub(crate) const DATA_TYPE_FLOAT64: u8 = 3;
// pub(crate) const DATA_TYPE_BYTES: u8 = 4;
pub(crate) const DATA_TYPE_UINT16: u8 = 5;
pub(crate) const DATA_TYPE_UINT32: u8 = 6;
pub(crate) const DATA_TYPE_MAP: u8 = 7;
pub(crate) const DATA_TYPE_INT32: u8 = 8;
pub(crate) const DATA_TYPE_UINT64: u8 = 9;
pub(crate) const DATA_TYPE_UINT128: u8 = 10;
pub(crate) const DATA_TYPE_SLICE: u8 = 11;
// pub(crate) const DATA_TYPE_DATA_CACHE_CONTAINER: u8 = 12;
// pub(crate) const DATA_TYPE_END_MARKER: u8 = 13;
pub(crate) const DATA_TYPE_BOOL: u8 = 14;
// pub(crate) const DATA_TYPE_FLOAT32: u8 = 15;

pub trait Decoder<'a>: Sized {
    fn decode(buf: &'a [u8], offset: &mut usize) -> Result<Self, Error> {
        let (data_type, size) = read_control(buf, offset)?;

        match data_type {
            DATA_TYPE_MAP => Self::decode_with_size(buf, offset, size),
            DATA_TYPE_POINTER => {
                let offset = &mut read_pointer(buf, offset, size)?;
                let (data_type, size) = read_control(buf, offset)?;
                match data_type {
                    // NOTE: The `offset` here is not the argument anymore.
                    DATA_TYPE_MAP => Self::decode_with_size(buf, offset, size),
                    _ => Err(Error::InvalidDataType(data_type)),
                }
            }
            _ => Err(Error::InvalidDataType(data_type)),
        }
    }

    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error>;
}

#[inline(always)]
pub(crate) fn read_control(buf: &[u8], offset: &mut usize) -> Result<(u8, usize), Error> {
    let control_byte = buf[*offset];
    *offset += 1;
    let mut data_type = control_byte >> 5;
    if data_type == DATA_TYPE_EXTENDED {
        data_type = buf[*offset] + 7;
        *offset += 1;
    }
    let mut size = (control_byte as usize) & 0x1f;
    if data_type == DATA_TYPE_EXTENDED || size < 29 {
        return Ok((data_type, size));
    }

    match size - 28 {
        1 => {
            if *offset > buf.len() - 1 {
                return Err(Error::InvalidOffset);
            }

            size = 29 + buf[*offset] as usize;
            *offset += 1;
        }
        2 => {
            if *offset > buf.len() - 2 {
                return Err(Error::InvalidOffset);
            }

            size = 285 + buf[*offset] as usize * 256 + buf[*offset + 1] as usize;
            *offset += 2;
        }
        n => {
            if *offset > buf.len() - n {
                return Err(Error::InvalidOffset);
            }

            size = 65_821;
            *offset += n;
        }
    }

    Ok((data_type, size))
}

#[inline]
pub(crate) fn read_str<'a>(buf: &'a [u8], offset: &mut usize) -> Result<&'a str, Error> {
    let (data_type, size) = read_control(buf, offset)?;
    let data = match data_type {
        DATA_TYPE_STRING => read_bytes(buf, offset, size)?,
        DATA_TYPE_POINTER => {
            let offset = &mut read_pointer(buf, offset, size)?;
            let (data_type, size) = read_control(buf, offset)?;
            match data_type {
                DATA_TYPE_STRING => read_bytes(buf, offset, size)?,
                _ => return Err(Error::InvalidDataType(data_type)),
            }
        }
        _ => return Err(Error::InvalidDataType(data_type)),
    };

    #[cfg(feature = "unsafe-str")]
    return Ok(unsafe { std::str::from_utf8_unchecked(data) });
    #[cfg(not(feature = "unsafe-str"))]
    std::str::from_utf8(data).map_err(Error::InvalidUtf8)
}

#[inline]
pub(crate) fn read_pointer(buf: &[u8], offset: &mut usize, size: usize) -> Result<usize, Error> {
    let pointer_size = ((size >> 3) & 0x3) + 1;
    let mut prefix = 0usize;
    if pointer_size != 4 {
        prefix = size & 0x7
    }

    let unpacked = {
        let mut value = prefix;
        for pos in *offset..*offset + pointer_size {
            value = value << 8 | buf[pos] as usize;
        }

        *offset += pointer_size;
        value
    };

    let pointer_value_offset = match pointer_size {
        2 => 2048,
        3 => 526_336,
        _ => 0,
    };
    Ok(unpacked + pointer_value_offset)
}

pub(crate) fn read_bool(buf: &[u8], offset: &mut usize) -> Result<bool, Error> {
    let (data_type, size) = read_control(buf, offset)?;

    match data_type {
        DATA_TYPE_BOOL => Ok(size != 0),
        DATA_TYPE_POINTER => {
            let offset = &mut read_pointer(buf, offset, size)?;
            let (data_type, size) = read_control(buf, offset)?;
            match data_type {
                DATA_TYPE_BOOL => Ok(size != 0),
                _ => Err(Error::InvalidDataType(data_type)),
            }
        }
        _ => Err(Error::InvalidDataType(data_type)),
    }
}

pub(crate) fn read_f64(buf: &[u8], offset: &mut usize) -> Result<f64, Error> {
    let (data_type, size) = read_control(buf, offset)?;

    #[inline(always)]
    fn bytes_to_f64(buf: &[u8]) -> f64 {
        let reserved: [u8; 8] = buf.try_into().unwrap();
        f64::from_be_bytes(reserved)
    }

    match data_type {
        DATA_TYPE_FLOAT64 => Ok(bytes_to_f64(read_bytes(buf, offset, size)?)),
        DATA_TYPE_POINTER => {
            let offset = &mut read_pointer(buf, offset, size)?;
            let (data_type, size) = read_control(buf, offset)?;
            match data_type {
                DATA_TYPE_FLOAT64 => Ok(bytes_to_f64(read_bytes(buf, offset, size)?)),
                _ => Err(Error::InvalidDataType(data_type)),
            }
        }
        _ => Err(Error::InvalidDataType(data_type)),
    }
}

pub(crate) fn read_usize(buf: &[u8], offset: &mut usize) -> Result<usize, Error> {
    let (data_type, size) = read_control(buf, offset)?;
    let size = match data_type {
        DATA_TYPE_UINT16 | DATA_TYPE_UINT32 | DATA_TYPE_INT32 | DATA_TYPE_UINT64
        | DATA_TYPE_UINT128 => size,
        DATA_TYPE_POINTER => {
            let offset = &mut read_pointer(buf, offset, size)?;
            let (data_type, size) = read_control(buf, offset)?;
            match data_type {
                DATA_TYPE_UINT16 | DATA_TYPE_UINT32 | DATA_TYPE_INT32 | DATA_TYPE_UINT64
                | DATA_TYPE_UINT128 => size,
                _ => return Err(Error::InvalidDataType(data_type)),
            }
        }
        _ => return Err(Error::InvalidDataType(data_type)),
    };

    if size == 0 {
        return Ok(0);
    }

    if *offset + size > buf.len() {
        return Err(Error::InvalidOffset);
    }

    let mut value = 0;
    for pos in *offset..*offset + size {
        let ch = buf[pos] as usize;

        value = value << 8 | ch;
    }

    *offset += size;

    Ok(value)
}

pub(crate) fn read_str_array<'a>(buf: &'a [u8], offset: &mut usize) -> Result<Vec<&'a str>, Error> {
    let (data_type, size) = read_control(buf, offset)?;
    match data_type {
        DATA_TYPE_SLICE => {
            let mut array = Vec::with_capacity(size);
            for _ in 0..size {
                array.push(read_str(buf, offset)?);
            }
            Ok(array)
        }
        DATA_TYPE_POINTER => {
            let offset = &mut read_pointer(buf, offset, size)?;
            let (data_type, size) = read_control(buf, offset)?;
            match data_type {
                DATA_TYPE_SLICE => {
                    let mut array = Vec::with_capacity(size);
                    for _ in 0..size {
                        array.push(read_str(buf, offset)?);
                    }
                    Ok(array)
                }
                _ => Err(Error::InvalidDataType(data_type)),
            }
        }
        _ => Err(Error::InvalidDataType(data_type)),
    }
}

pub(crate) fn read_map<'a>(
    buf: &'a [u8],
    offset: &mut usize,
) -> Result<Vec<(&'a str, &'a str)>, Error> {
    let (data_type, size) = read_control(buf, offset)?;

    match data_type {
        DATA_TYPE_MAP => {
            let mut map = Vec::with_capacity(size);
            for _ in 0..size {
                map.push((read_str(buf, offset)?, read_str(buf, offset)?));
            }
            Ok(map)
        }
        DATA_TYPE_POINTER => {
            let offset = &mut read_pointer(buf, offset, size)?;
            let (data_type, size) = read_control(buf, offset)?;
            match data_type {
                DATA_TYPE_MAP => {
                    let mut map = Vec::with_capacity(size);
                    for _ in 0..size {
                        map.push((read_str(buf, offset)?, read_str(buf, offset)?));
                    }
                    Ok(map)
                }
                _ => Err(Error::InvalidDataType(data_type)),
            }
        }
        _ => Err(Error::InvalidDataType(data_type)),
    }
}

#[inline]
fn read_bytes<'a>(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<&'a [u8], Error> {
    let new_offset = *offset + size;
    if new_offset > buf.len() {
        return Err(Error::InvalidOffset);
    }
    let bytes = &buf[*offset..new_offset];
    *offset = new_offset;
    Ok(bytes)
}

#[inline]
pub(crate) fn bytes_to_usize(buf: &[u8]) -> usize {
    let mut value = 0usize;
    for &b in buf {
        value = value << 8 | b as usize
    }

    value
}

#[inline]
pub(crate) fn bytes_to_usize_with_prefix(prefix: usize, buf: &[u8]) -> usize {
    match buf.len() {
        0..=8 => {
            let mut value = prefix;
            for &b in buf {
                value = value << 8 | b as usize
            }

            value
        }
        _ => 0,
    }
}
