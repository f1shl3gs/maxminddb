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
                    DATA_TYPE_MAP => Self::decode_with_size(buf, offset, size),
                    _ => Err(Error::InvalidDataType(data_type)),
                }
            }
            _ => Err(Error::InvalidDataType(data_type)),
        }
    }

    fn decode_with_size(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<Self, Error>;
}

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
    let bytes_to_read = size - 28;
    size = bytes_to_usize(read_bytes(buf, offset, bytes_to_read)?);
    size += match bytes_to_read {
        1 => 29,
        2 => 285,
        _ => 65_821,
    };
    Ok((data_type, size))
}

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
    std::str::from_utf8(data)
}

pub(crate) fn read_pointer(buf: &[u8], offset: &mut usize, size: usize) -> Result<usize, Error> {
    let pointer_size = ((size >> 3) & 0x3) + 1;
    let mut prefix = 0usize;
    if pointer_size != 4 {
        prefix = size & 0x7
    }
    let unpacked = bytes_to_usize_with_prefix(prefix, read_bytes(buf, offset, pointer_size)?);
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

    match data_type {
        DATA_TYPE_FLOAT64 => Ok(f64::from_bits(
            bytes_to_usize(read_bytes(buf, offset, size)?) as u64,
        )),
        DATA_TYPE_POINTER => {
            let offset = &mut read_pointer(buf, offset, size)?;
            let (data_type, size) = read_control(buf, offset)?;
            match data_type {
                DATA_TYPE_FLOAT64 => Ok(f64::from_bits(bytes_to_usize(read_bytes(
                    buf, offset, size,
                )?) as u64)),
                _ => Err(Error::InvalidDataType(data_type)),
            }
        }
        _ => Err(Error::InvalidDataType(data_type)),
    }
}

pub(crate) fn read_usize(buf: &[u8], offset: &mut usize) -> Result<usize, Error> {
    let (data_type, size) = read_control(buf, offset)?;
    match data_type {
        DATA_TYPE_UINT16 | DATA_TYPE_UINT32 | DATA_TYPE_INT32 | DATA_TYPE_UINT64
        | DATA_TYPE_UINT128 => Ok(bytes_to_usize(read_bytes(buf, offset, size)?)),
        DATA_TYPE_POINTER => {
            let offset = &mut read_pointer(buf, offset, size)?;
            let (data_type, size) = read_control(buf, offset)?;
            match data_type {
                DATA_TYPE_UINT16 | DATA_TYPE_UINT32 | DATA_TYPE_INT32 | DATA_TYPE_UINT64
                | DATA_TYPE_UINT128 => Ok(bytes_to_usize(read_bytes(buf, offset, size)?)),
                _ => Err(Error::InvalidDataType(data_type)),
            }
        }
        _ => Err(Error::InvalidDataType(data_type)),
    }
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

fn read_bytes<'a>(buf: &'a [u8], offset: &mut usize, size: usize) -> Result<&'a [u8], Error> {
    let new_offset = *offset + size;
    if new_offset > buf.len() {
        return Err(Error::InvalidOffset);
    }
    let bytes = &buf[*offset..new_offset];
    *offset = new_offset;
    Ok(bytes)
}

fn bytes_to_usize(buf: &[u8]) -> usize {
    match buf.len() {
        1 => buf[0] as usize,
        2 => (buf[0] as usize) << 8 | (buf[1] as usize),
        3 => ((buf[0] as usize) << 8 | (buf[1] as usize)) << 8 | (buf[2] as usize),
        4 => {
            (((buf[0] as usize) << 8 | (buf[1] as usize)) << 8 | (buf[2] as usize)) << 8
                | (buf[3] as usize)
        }
        5 => {
            ((((buf[0] as usize) << 8 | (buf[1] as usize)) << 8 | (buf[2] as usize)) << 8
                | (buf[3] as usize))
                << 8
                | (buf[4] as usize)
        }
        6 => {
            (((((buf[0] as usize) << 8 | (buf[1] as usize)) << 8 | (buf[2] as usize)) << 8
                | (buf[3] as usize))
                << 8
                | (buf[4] as usize))
                << 8
                | (buf[5] as usize)
        }
        7 => {
            ((((((buf[0] as usize) << 8 | (buf[1] as usize)) << 8 | (buf[2] as usize)) << 8
                | (buf[3] as usize))
                << 8
                | (buf[4] as usize))
                << 8
                | (buf[5] as usize))
                << 8
                | (buf[6] as usize)
        }
        8 => {
            (((((((buf[0] as usize) << 8 | (buf[1] as usize)) << 8 | (buf[2] as usize)) << 8
                | (buf[3] as usize))
                << 8
                | (buf[4] as usize))
                << 8
                | (buf[5] as usize))
                << 8
                | (buf[6] as usize))
                << 8
                | (buf[7] as usize)
        }
        _ => 0,
    }
}

fn bytes_to_usize_with_prefix(prefix: usize, buf: &[u8]) -> usize {
    match buf.len() {
        0 => prefix,
        1 => prefix << 8 | (buf[0] as usize),
        2 => (prefix << 8 | (buf[0] as usize)) << 8 | (buf[1] as usize),
        3 => ((prefix << 8 | (buf[0] as usize)) << 8 | (buf[1] as usize)) << 8 | (buf[2] as usize),
        4 => {
            (((prefix << 8 | (buf[0] as usize)) << 8 | (buf[1] as usize)) << 8 | (buf[2] as usize))
                << 8
                | (buf[3] as usize)
        }
        5 => {
            ((((prefix << 8 | (buf[0] as usize)) << 8 | (buf[1] as usize)) << 8
                | (buf[2] as usize))
                << 8
                | (buf[3] as usize))
                << 8
                | (buf[4] as usize)
        }
        6 => {
            (((((prefix << 8 | (buf[0] as usize)) << 8 | (buf[1] as usize)) << 8
                | (buf[2] as usize))
                << 8
                | (buf[3] as usize))
                << 8
                | (buf[4] as usize))
                << 8
                | (buf[5] as usize)
        }
        7 => {
            ((((((prefix << 8 | (buf[0] as usize)) << 8 | (buf[1] as usize)) << 8
                | (buf[2] as usize))
                << 8
                | (buf[3] as usize))
                << 8
                | (buf[4] as usize))
                << 8
                | (buf[5] as usize))
                << 8
                | (buf[6] as usize)
        }
        8 => {
            (((((((prefix << 8 | (buf[0] as usize)) << 8 | (buf[1] as usize)) << 8
                | (buf[2] as usize))
                << 8
                | (buf[3] as usize))
                << 8
                | (buf[4] as usize))
                << 8
                | (buf[5] as usize))
                << 8
                | (buf[6] as usize))
                << 8
                | (buf[7] as usize)
        }
        _ => 0,
    }
}
