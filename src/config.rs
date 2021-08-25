use std::collections::HashMap;
use std::convert::TryInto;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use crate::linalg::Vector3;
use crate::shapes::{Plane, Ray, Shape, Sphere};
use crate::trace::{Color, Material, Object};

#[derive(Debug)]
pub enum ConfigError {
    ImageError(image::ImageError),
    IOError(std::io::Error),
    InvalidShape(String),
    InvalidObject(String),
    InvalidLine(String),
    NotEnoughLines
}

pub type ConfigResult<Ret> = Result<Ret, ConfigError>;

pub struct Config {
    pub objects: Vec<Object>, 
    pub pov: Ray, 
    pub width: u32,
    pub height: u32, 
    pub fov: f64,
    pub max_depth: u16,
    pub num_tries: u16,
    pub max_variation: f64
}

trait FromString: Shape {
    fn name() -> String;
    fn from_string(parts: &[&str]) -> Box<dyn Shape>;
}

impl FromString for Sphere {
    fn name() -> String {
        "sphere".to_string()
    }

    fn from_string(parts: &[&str]) -> Box<dyn Shape> {
        if parts.len() != 4 {
            panic!("Invalid configuration for sphere: {:?}", parts);
        }

        let parts: Vec<_> = parts.iter().map(|part| part.parse().unwrap()).collect();

        Box::new(Sphere { 
            center: Vector3::new(parts[0], parts[1], parts[2]), 
            radius: parts[3]
        })
    }
}

impl FromString for Plane {
    fn name() -> String {
        "plane".to_string()
    }

    fn from_string(parts: &[&str]) -> Box<dyn Shape> {
        if parts.len() != 6 {
            panic!("Invalid configuration for sphere: {:?}", parts);
        }

        let parts: Vec<_> = parts.iter().map(|part| part.parse().unwrap()).collect();

        Box::new(Plane { 
            point: Vector3::new(parts[0], parts[1], parts[2]), 
            norm: Vector3::new(parts[3], parts[4], parts[5]).normalize()
        })
    }
}

fn parse_nums<T: FromStr, const N: usize>(line: &str) -> ConfigResult<[T; N]> {
    let err = || ConfigError::InvalidLine(line.to_string());
    line.split(" ")
        .map(|num| num.parse::<T>().map_err(|_| err()))
        .collect::<ConfigResult<Vec<_>>>()?
        .try_into()
        .map_err(|_| err())
}

fn parse_vec(line: &str) -> ConfigResult<Vector3> {
    let [x, y, z] = parse_nums(line)?;
    Ok(Vector3::new(x, y, z))
}

fn parse_shape(parts: &[&str]) -> ConfigResult<Box<dyn Shape>> {
    let fail = || {
        let fail_str = parts.iter().cloned().collect::<Vec<_>>().join(" ");
        ConfigError::InvalidShape(fail_str)
    };

    let mut parts = parts.iter().cloned().filter(|part| *part != "");

    let shape_name = parts.next().ok_or_else(fail)?;
    let rest_parts: Vec<_> = parts.collect();

    let shape_parsers: HashMap<_, _> = {
        let pairs: [(String, &dyn Fn(&[&str]) -> Box<dyn Shape>); 2] = [
            (Sphere::name(), &Sphere::from_string),
            (Plane::name(), &Plane::from_string),
        ];
        pairs.iter().cloned().collect()
    };

    let parser = shape_parsers.get(shape_name).ok_or_else(fail)?;
    Ok((parser)(&rest_parts))
}

fn parse_object(raw: &str, col_scale: f64, lum_scale: f64) -> ConfigResult<Object> {
    let fail = || {
        let fail_str = raw.to_string();
        ConfigError::InvalidObject(fail_str)
    };
    
    let mut parts = raw.split(" ");

    let color = {
        let color_str = parts.next().ok_or_else(fail)?;
        Color::from_string(color_str)
            .ok_or_else(fail)?
            .scale(col_scale)
    };
    let lum = {
        let lum_const: f64 = parts.next().ok_or_else(fail)?.parse().map_err(|_| fail())?;
        color.scale(lum_const).scale(lum_scale / col_scale)
    };
    let material = match parts.next().ok_or_else(fail)? {
        "mirror" => Material::Mirror,
        "glass" => Material::Translucent(1.0),
        "opaque" => Material::Translucent(0.0),
        "translucent" => {
            let clearness = parts.next().ok_or_else(fail)?;
            let clearness = clearness.parse().map_err(|_| fail())?;
            Material::Translucent(clearness)
        }
        _ => return Err(fail())
    };
    
    let shape = parse_shape(&parts.collect::<Vec<_>>())?;
    Ok(Object { shape, color, lum, material })
}

fn parse_pov(pos_line: &str, dir_line: &str) -> ConfigResult<Ray> {
    let pos = parse_vec(pos_line)?;
    let dir = parse_vec(dir_line)?;
    Ok(Ray::new(pos, dir))
}


pub fn parse_config(raw: &str) -> ConfigResult<Config> {
    let mut lines = raw
        .split("\n")
        .filter(|line| *line != "")
        .filter(|line| !line.starts_with("//"));
    let mut next_line = || lines.next().ok_or_else(|| ConfigError::NotEnoughLines);
    
    let pov = parse_pov(next_line()?, next_line()?)?;
    let [width, height] = parse_nums(next_line()?)?;
    let [fov] = parse_nums(next_line()?)?;
    let [max_depth, num_tries] = parse_nums(next_line()?)?;
    let [max_variation] = parse_nums(next_line()?)?;

    let [col_scale, lum_scale] = parse_nums(next_line()?)?;
    let objects: Vec<_> = lines
        .map(|line| parse_object(line, col_scale, lum_scale))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Config { 
        objects,
        pov,
        width,
        height,
        fov,
        max_depth,
        num_tries,
        max_variation
    })
}

pub fn parse_config_file(path: &PathBuf) -> ConfigResult<Config> {
    fs::read_to_string(path)
        .map_err(ConfigError::IOError)
        .and_then(|contents| parse_config(&contents))
}