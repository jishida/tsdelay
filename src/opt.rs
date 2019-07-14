use std::fmt;
use std::num::ParseIntError;
use std::path::{Path, PathBuf};
use structopt::{clap, StructOpt};

const MILLI: &str = "milli";
const MICRO: &str = "micro";
const INT_MILLI: &str = "int-milli";
const INT_MICRO: &str = "int-micro";
const REAL: &str = "real";
const REAL_MILLI: &str = "real-milli";
const REAL_MICRO: &str = "real-micro";
const RAW: &str = "raw";

#[derive(Debug)]
pub enum Format {
    Milli,
    Micro,
    Real,
    RealMilli,
    RealMicro,
    Raw,
}

impl std::str::FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Format, String> {
        match s.to_lowercase().as_str() {
            MILLI | INT_MILLI => Ok(Format::Milli),
            MICRO | INT_MICRO => Ok(Format::Micro),
            REAL => Ok(Format::Real),
            REAL_MILLI => Ok(Format::RealMilli),
            REAL_MICRO => Ok(Format::RealMicro),
            RAW => Ok(Format::Raw),
            _ => Err(format!(
                "possible values: {}",
                Format::variants().join(", ")
            )),
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl Format {
    fn to_str(&self) -> &'static str {
        match self {
            Format::Milli => MILLI,
            Format::Micro => MICRO,
            Format::Real => REAL,
            Format::RealMilli => REAL_MILLI,
            Format::RealMicro => REAL_MICRO,
            Format::Raw => RAW,
        }
    }

    fn variants() -> Vec<&'static str> {
        vec![MILLI, MICRO, REAL, REAL_MILLI, REAL_MICRO, RAW]
    }
}

fn parse_pid(s: &str) -> Result<u16, ParseIntError> {
    let pid = if s.starts_with("0x") {
        u16::from_str_radix(&s[2..], 16)?
    } else {
        u16::from_str_radix(s, 10)?
    };
    if (pid & 0xe000) == 0 {
        Ok(pid)
    } else {
        u16::from_str_radix("10000", 16)
    }
}

#[derive(StructOpt, Debug)]
#[structopt(
    name = "tsdelay",
    author = "",
    raw(setting = "clap::AppSettings::DeriveDisplayOrder")
)]
pub struct Opt {
    /// Selects display format
    #[structopt(
        name = "FORMAT",
        short = "f",
        long = "format",
        default_value = "milli",
        raw(possible_values = "&Format::variants()", case_insensitive = "true")
    )]
    format: Format,
    /// Specifies video PID
    #[structopt(
        name = "VIDEO PID",
        short = "v",
        long = "video",
        parse(try_from_str = "parse_pid")
    )]
    video_id: Option<u16>,
    /// Specifies audio PID
    #[structopt(
        name = "AUDIO PID",
        short = "a",
        long = "audio",
        parse(try_from_str = "parse_pid")
    )]
    audio_id: Option<u16>,
    /// Drops broken audio packets
    #[structopt(short = "d", long = "drop-broken-audio")]
    drop_broken_audio: bool,
    #[structopt(name = "SOURCE", parse(from_os_str))]
    source: PathBuf,
}

impl Opt {
    pub fn video_id(&self) -> Option<u16> {
        self.video_id
    }

    pub fn audio_id(&self) -> Option<u16> {
        self.audio_id
    }

    pub fn source(&self) -> &Path {
        self.source.as_path()
    }

    pub fn drop_broken_audio(&self) -> bool {
        self.drop_broken_audio
    }

    pub fn numerator(&self) -> i32 {
        match self.format {
            Format::Milli | Format::RealMilli => 1_000i32,
            Format::Micro | Format::RealMicro => 1_000_000i32,
            _ => 1i32,
        }
    }

    pub fn denominator(&self) -> i32 {
        match self.format {
            Format::Raw => 1i32,
            _ => 90_000i32,
        }
    }

    pub fn is_real(&self) -> bool {
        match self.format {
            Format::Real | Format::RealMicro | Format::RealMilli => true,
            _ => false,
        }
    }
}
