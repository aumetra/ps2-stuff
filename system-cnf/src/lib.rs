#![doc = include_str!("../README.md")]
#![forbid(missing_docs, unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]

use std::{
    borrow::Cow,
    error::Error as StdError,
    fmt::{self, Display},
    str::FromStr,
};

/// Errors that might occurr when
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Error {
    /// Malformed `SYSTEM.CNF`
    MalformedFile,

    /// Required field is missing
    MissingField,

    /// Video mode is unknown
    UnknownVideoMode,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl StdError for Error {}

/// Video mode of the ROM
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum VideoMode {
    /// NTSC
    Ntsc,

    /// PAL
    Pal,
}

impl VideoMode {
    /// Retrieve the value of the video mode in its string representation
    fn as_str(&self) -> &str {
        match self {
            Self::Ntsc => "NTSC",
            Self::Pal => "PAL",
        }
    }
}

impl FromStr for VideoMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "NTSC" => Ok(Self::Ntsc),
            "PAL" => Ok(Self::Pal),
            _ => Err(Error::UnknownVideoMode),
        }
    }
}

/// Parsed form of a `SYSTEM.CNF` file
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SystemCnf<'a> {
    /// Path to the initial ELF file
    pub elf_path: Cow<'a, str>,

    /// Version of the game
    pub version: Cow<'a, str>,

    /// Video mode
    pub video_mode: VideoMode,

    /// ???
    pub hdd_unit_power: Option<Cow<'a, str>>,
}

impl<'a> SystemCnf<'a> {
    /// Parse a `SYSTEM.CNF` file
    ///
    /// # Errors
    ///
    /// - The video mode is invalid
    /// - Required fields are missing
    /// - The file is somehow malformed
    pub fn parse(raw_cnf: &'a str) -> Result<Self, Error> {
        // Not really a fan of this parsing approach but I can't think of anything better ATM
        let mut elf_path = None;
        let mut version = None;
        let mut video_mode = None;
        let mut hdd_unit_power = None;

        for line in raw_cnf.lines() {
            let mut kv_iter = line.split('=');
            let key = kv_iter.next().ok_or(Error::MalformedFile)?;
            let value = kv_iter.next().ok_or(Error::MalformedFile)?;

            match key.trim() {
                "BOOT2" => {
                    let mut path = value.trim();
                    path = path.strip_suffix(";1").unwrap_or(path);
                    elf_path = Some(path.into());
                }
                "VER" => version = Some(value.trim().into()),
                "VMODE" => video_mode = Some(value.parse()?),
                "HDDUNITPOWER" => hdd_unit_power = Some(value.trim().into()),
                _ => (),
            }
        }

        Ok(Self {
            elf_path: elf_path.ok_or(Error::MissingField)?,
            version: version.ok_or(Error::MissingField)?,
            video_mode: video_mode.ok_or(Error::MissingField)?,
            hdd_unit_power,
        })
    }
}

impl Display for SystemCnf<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BOOT2 = {};1\r\nVER = {}\r\nVMODE = {}\r\n",
            self.elf_path,
            self.version,
            self.video_mode.as_str()
        )?;
        if let Some(ref hdd_unit_power) = self.hdd_unit_power {
            write!(f, "HDDUNITPOWER = {}\r\n", hdd_unit_power)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{SystemCnf, VideoMode};
    use std::str;

    static SYSTEM_CNF: &[u8] = &[
        0x42, 0x4F, 0x4F, 0x54, 0x32, 0x20, 0x3D, 0x20, 0x63, 0x64, 0x72, 0x6F, 0x6D, 0x30, 0x3A,
        0x5C, 0x53, 0x4C, 0x55, 0x53, 0x5F, 0x32, 0x31, 0x33, 0x2E, 0x34, 0x38, 0x3B, 0x31, 0x0D,
        0x0A, 0x56, 0x45, 0x52, 0x20, 0x3D, 0x20, 0x31, 0x2E, 0x30, 0x30, 0x0D, 0x0A, 0x56, 0x4D,
        0x4F, 0x44, 0x45, 0x20, 0x3D, 0x20, 0x4E, 0x54, 0x53, 0x43, 0x0D, 0x0A,
    ];

    #[test]
    fn encode() {
        let txt = str::from_utf8(SYSTEM_CNF).unwrap();
        let parsed = SystemCnf::parse(txt).unwrap();
        let encoded = parsed.to_string();

        assert_eq!(txt, encoded);
        assert_eq!(SYSTEM_CNF, encoded.as_bytes());
    }

    #[test]
    fn decode() {
        let txt = str::from_utf8(SYSTEM_CNF).unwrap();
        let parsed = SystemCnf::parse(txt).unwrap();

        assert_eq!(parsed.elf_path, "cdrom0:\\SLUS_213.48");
        assert_eq!(parsed.version, "1.00");
        assert_eq!(parsed.video_mode, VideoMode::Ntsc);
        assert_eq!(parsed.hdd_unit_power, None);
    }
}
