mod config;
mod linalg;
mod shapes;
mod trace;


extern crate image;
extern crate rand;
extern crate rayon;
extern crate itertools;

use crate::linalg::Vector3;
use crate::config::{Config, ConfigError, ConfigResult, parse_config_file};
use crate::trace::make_image;

use config::parse_config;
use image::{ImageBuffer, Rgb};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::thread::sleep;
use std::time::Duration;
use structopt::StructOpt;

static PREV_LEN: AtomicUsize = AtomicUsize::new(0);

macro_rules! message {
    ($($items:tt)*) => {{
        use core::sync::atomic::Ordering;
        let message = format!($($items)*);
        let num_erase = PREV_LEN.swap(message.len(), Ordering::Relaxed);
        print!("\r{}", vec![" "; num_erase].join(""));
        print!("\r{}", message);
        std::io::stdout().flush().map_err(ConfigError::IOError)?;
    }}
}

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct CliArgs {
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    #[structopt(parse(from_os_str))]
    output: PathBuf,

    #[structopt(short, long)]
    real_time: bool
}

fn main() -> ConfigResult<()> {
    let cli_args = CliArgs::from_args();

    if cli_args.real_time {
        build_real_time(&cli_args.input, &cli_args.output)
    } else {
        build_once(&cli_args.input, &cli_args.output)
    }
}

fn build_once(input: &PathBuf, output: &PathBuf) -> ConfigResult<()> {
    let config = parse_config_file(input)?;
    let result = make_image(&config);
    let img = ImageBuffer::from_fn(config.width, config.height, |x, y| {
        let curr = result[y as usize][x as usize];
        Rgb([curr.x as u8, curr.y as u8, curr.z as u8])
    });

    img.save(output).map_err(ConfigError::ImageError)
}

fn build_real_time(input: &PathBuf, output: &PathBuf) -> ConfigResult<()> {
    fn get_config(input: &PathBuf, cached: Option<&str>) -> ConfigResult<Option<(String, Config)>> {
        let load_raw = || std::fs::read_to_string(input).map_err(ConfigError::IOError);
        let mut raw = load_raw()?;
        if cached == Some(&raw) {
            return Ok(None);
        }

        loop {
            match parse_config(&raw) {
                Ok(config) => return Ok(Some((raw, config))),
                Err(err) => {
                    message!("Config Error: {:?}", err);
                    raw = loop {
                        let new_raw = load_raw()?;
                        if new_raw != raw {
                            break new_raw;
                        }
                        sleep(Duration::from_secs(1));
                    };
                }
            }
        }
    }

    fn empty_result(config: &Config) -> Vec<Vec<Vector3>> {
        vec![vec![Vector3::new(0.0, 0.0, 0.0); config.width as usize]; config.height as usize]
    }

    let (mut raw, mut config) = get_config(input, None)?.unwrap();
    let mut result = empty_result(&config);
    for it in 1.. {
        message!("\rIter #{}", it);
        std::io::stdout().flush().map_err(ConfigError::IOError)?;

        {
            let new = make_image(&config);
            for x in 0..(config.width as usize) {
                for y in 0..(config.height as usize) {
                    result[y][x] = result[y][x] + new[y][x];
                }
            }
        }

        {
            let img = ImageBuffer::from_fn(config.width, config.height, |x, y| {
                let curr = result[y as usize][x as usize].scale(1.0 / (it as f64));
                Rgb([curr.x as u8, curr.y as u8, curr.z as u8])
            });
        
            img.save(output).map_err(ConfigError::ImageError)?;
        }

        match get_config(input, Some(&raw))? {
            None => (),
            Some((new_raw, new_config)) => {
                raw = new_raw;
                config = new_config;
                result = empty_result(&config);
            }
        }
    }

    Ok(())
}
