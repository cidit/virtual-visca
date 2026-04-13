use anyhow::{self};
use grafton_visca::{
    command::bytes::constants::{self, pan_tilt},
    types::{PanSpeed, TiltSpeed},
};
use itertools::Itertools;
use thiserror::Error;

use crate::visca;

pub enum CategoryCode {
    Command,
    Inquiry,
}

#[derive(Error, Debug)]
#[error("Unrecognized category code: {0:?}")]
pub struct UnrecognizedCategoryError(u8);

impl TryFrom<&u8> for CategoryCode {
    type Error = UnrecognizedCategoryError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::Command),
            0x09 => Ok(Self::Inquiry),
            other => Err(UnrecognizedCategoryError(*other)),
        }
    }
}

pub enum CommandCode {
    PanTilt,
    Zoom,
}

#[derive(Error, Debug)]
#[error("Unrecognized command code: {0:?}")]
pub struct UnrecognizedCommandCode(u8);

impl TryFrom<&u8> for CommandCode {
    type Error = UnrecognizedCommandCode;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0x06 => Ok(Self::PanTilt),
            // TODO: FIND_CODE => Ok(Zoom),
            other => Err(UnrecognizedCommandCode(*other)),
        }
    }
}

use grafton_visca::command as gvc;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error(transparent)]
    UnrecognizedCategory(#[from] UnrecognizedCategoryError),
    #[error(transparent)]
    UnrecognizedCommand(#[from] UnrecognizedCommandCode),

    #[error("Badly formed data: [{0:?}]")]
    BadlyFormed(Vec<u8>),

    #[error(transparent)]
    Raw(#[from] anyhow::Error),

    #[error("Unimplemented feature: {0:?}")]
    Unimplemented(String),

    #[error("Decode ended unexpectedly")]
    Incomplete,

    #[error("Missing Terminator")]
    NoTerminator,
}
fn ptd_from_u8s(pan_d: u8, tilt_d: u8) -> Result<gvc::PanTiltDirection, DecodeError> {
    match (pan_d, tilt_d) {
        (0x03, 0x01) => Ok(gvc::PanTiltDirection::Up),
        (0x03, 0x02) => Ok(gvc::PanTiltDirection::Down),
        (0x01, 0x03) => Ok(gvc::PanTiltDirection::Left),
        (0x02, 0x03) => Ok(gvc::PanTiltDirection::Right),
        (0x01, 0x01) => Ok(gvc::PanTiltDirection::UpLeft),
        (0x02, 0x01) => Ok(gvc::PanTiltDirection::UpRight),
        (0x01, 0x02) => Ok(gvc::PanTiltDirection::DownLeft),
        (0x02, 0x02) => Ok(gvc::PanTiltDirection::DownRight),
        (0x03, 0x03) => Ok(gvc::PanTiltDirection::Stop),
        _ => Err(anyhow::anyhow!("invalid pan tilt direction codes"))?,
    }
}

pub fn decode(buf: &[u8]) -> Result<visca::Command, DecodeError> {
    let mut itr = buf.into_iter();

    let Some(_id) = itr.next() else {
        Err(anyhow::anyhow!("expected an address byte at least."))?
    };

    let Some(last) = itr.next_back() else {
        return Err(DecodeError::Incomplete);
    };

    if *last != 0xFF {
        return Err(DecodeError::NoTerminator);
    }

    match itr
        .next()
        .ok_or(DecodeError::Incomplete)
        .map(CategoryCode::try_from)??
    {
        CategoryCode::Command => match itr
            .next()
            .ok_or(DecodeError::Incomplete)
            .map(CommandCode::try_from)??
        {
            CommandCode::PanTilt => match itr.next().ok_or(DecodeError::Incomplete)? {
                0x04 => Ok(visca::Command::PanTilt(gvc::PanTilt::Home)),
                0x05 => Ok(visca::Command::PanTilt(gvc::PanTilt::Reset)),
                0x01 => {
                    let Some((pan_s, tilt_s, pan_d, tilt_d)) = itr.take(3).collect_tuple() else {
                        return Err(DecodeError::BadlyFormed(buf.to_vec()));
                    };
                    let direction = ptd_from_u8s(*pan_d, *tilt_d)?;
                    Ok(visca::Command::PanTilt(gvc::PanTilt::Move {
                        direction,
                        pan_speed: PanSpeed::new(*pan_s).unwrap(),
                        tilt_speed: TiltSpeed::new(*tilt_s).unwrap(),
                    }))
                }
                _ => Err(anyhow::anyhow!("unexpected pantilt subcommand"))?,
            },
            CommandCode::Zoom => Err(DecodeError::Unimplemented("Zoom".to_string())),
        },
        CategoryCode::Inquiry => Err(DecodeError::Unimplemented("Inquiry".to_string())),
    }
}

pub trait DecodeVisca
where
    Self: Sized,
{
    fn try_decode(buf: &[u8]) -> Result<Self, DecodeError>;
    fn decode_from(buf: &[u8]) -> Self {
        Self::try_decode(buf).unwrap()
    }
}

impl DecodeVisca for grafton_visca::command::PanTilt {
    fn try_decode(buf: &[u8]) -> Result<Self, DecodeError> {
        fn match_prefix<'a>(buf: &'a [u8], prefix: &'a [u8]) -> Option<&'a [u8]> {
            if buf[1..prefix.len()] == prefix[1..] {
                Some(&buf[prefix.len()..])
            } else {
                None
            }
        }

        if let Some(_) = match_prefix(buf, pan_tilt::HOME) {
            return Ok(Self::Home);
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
