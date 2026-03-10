use std::io::Read;

use grafton_visca::command::EncodeVisca;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("Unrecognized command code: {0:?}")]
    Unrecognized([u8; 4]),
    #[error("Unimplemented command: {0}")]
    Unimplemented(String),
    #[error("Badly formed data: [{0:?}]")]
    BadlyFormed(Vec<u8>),
}

pub fn decode(buf: &[u8]) -> Result<(), DecodeError> {
    let mut itr = buf.iter();
    let Some(id) = itr.next() else {
        return Err(DecodeError::BadlyFormed(buf.to_vec()))
    };

    let Some(categorty) = itr.next() else {
        return Err(DecodeError::BadlyFormed(buf.to_vec()))
    };

    match categorty {
        0x01 => unimplemented!(),
        0x09 => unimplemented!(),
        o => return Err(DecodeError::Unrecognized([*categorty, 0x00, 0x00, 0x00].to_owned()))
    }
    
}

pub trait DecodeVisca
where
    Self: EncodeVisca + Sized,
{
    fn try_decode(buf: &[u8]) -> Result<Self, DecodeError>;
    fn decode_from(buf: &[u8]) -> Self {
        Self::try_decode(buf).unwrap()
    }
}

impl DecodeVisca for grafton_visca::command::PanTilt {
    fn try_decode(buf: &[u8]) -> Result<Self, DecodeError> {
        use grafton_visca::command::const_encoding::constants::pan_tilt;

        fn match_prefix<'a>(buf: &'a [u8], prefix: &'a [u8]) -> Option<&'a [u8]> {
            if buf[1..prefix.len()] == prefix[1..] {
                Some(&buf[prefix.len()..])
            } else {
                None
            }
        }

        if let Some(_) = match_prefix(buf, pan_tilt::HOME) {
            return Ok(Self::Home)
        }
        if let Some(_) = match_prefix(buf, pan_tilt::RESET) {
            return Ok(Self::Reset);
        }
        unimplemented!()
        // Err(DecodeError::Unrecognized(buf))
        // match buf {
        //     pan_tilt::HOME => Ok(Self::Home),
        //     pan_tilt::RESET => Ok(Self::Reset),
        //     pan_tilt::MOVE_PREFIX => unimplemented!(),
        //     pan_tilt::ABSOLUTE_PREFIX => unimplemented!(),
        //     pan_tilt::RELATIVE_PREFIX => unimplemented!(),
        //     pan_tilt::LIMIT_SET_PREFIX => unimplemented!(),
        //     pan_tilt::LIMIT_CLEAR_PREFIX => unimplemented!(),
        //     other => Err(DecodeError::Unrecognized(other)),
        // }
    }
}
