extern crate ffmpeg_sys as ffi;
extern crate libc;
extern crate structopt;

mod error;
mod opt;
mod util;

use self::error::Result;
use self::opt::Opt;
use std::error::Error;
use structopt::StructOpt;

fn main() {
    let opt = Opt::from_args();
    if let Err(ref e) = print_delay(&opt) {
        eprintln!("tsdelay: Failed to get delay");
        eprintln!("error: {}", e);
        if let Some(err) = e.source() {
            eprintln!("cause: {}", err);
        }
    }
}

fn print_delay(opt: &Opt) -> Result<()> {
    util::init_ffmpeg();
    let raw_delay = util::get_delay(&opt)?;
    let delay_numerator = raw_delay * opt.numerator() as i64;
    if opt.is_real() {
        println!("{}", delay_numerator as f64 / opt.denominator() as f64);
    } else {
        println!("{}", delay_numerator / opt.denominator() as i64);
    }
    Ok(())
}
