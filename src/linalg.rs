use std::ops::{Add, Sub, Mul, Div};
use rand::Rng;

#[derive(Debug, Copy, Clone)]
pub struct Vector3 {
    pub x: f64, pub y: f64, pub z: f64,
    pub rho: f64, pub theta: f64, pub phi: f64
}

impl Vector3 {

    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self::new_xyz(x, y, z)
    }

    pub fn new_xyz(x: f64, y: f64, z: f64) -> Self {
        let rho = (x.powi(2) + y.powi(2) + z.powi(2)).sqrt();
        let theta = y.atan2(x);
        let phi = if rho == 0.0 { 0.0 } else { (z / rho).acos() };
        Self { x, y, z, rho, theta, phi }
    }

    pub fn new_sph(rho: f64, theta: f64, phi: f64) -> Self {
        let x = rho * theta.cos() * phi.sin();
        let y = rho * theta.sin() * phi.sin();
        let z = rho * phi.cos();
        Self { x, y, z, rho, theta, phi }
    }

    pub fn rand_hemi() -> Self {
        let mut rng = rand::thread_rng();
        let u1 = rng.gen::<f64>();
        let u2 = rng.gen::<f64>();
        
        let r = u1.sqrt();
        let theta = 2.0 * std::f64::consts::PI * u2;
    
        let x = r * theta.cos();
        let y = r * theta.sin();
    
        Self::new(x, y, (1.0 - u1).sqrt()).normalize()
    }

    pub fn rand_hemi2() -> Self {
        let mut rng = rand::thread_rng();
        let u1 = rng.gen::<f64>();
        let u2 = rng.gen::<f64>();

        let r = (1.0 - u1.powi(2)).sqrt();
        let phi = 2.0 * std::f64::consts::PI * u2;
        Vector3::new(r * phi.cos(), r * phi.sin(), u1)
    }

    pub fn dot(&self, other: Self) -> f64 {
        (self.x * other.x) + (self.y * other.y) + (self.z * other.z)
    }

    pub fn cross(&self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x
        )
    }

    pub fn scale(&self, scale: f64) -> Self {
        Self::new(
            self.x * scale, self.y * scale, self.z * scale
        )
    }

    pub fn size(&self) -> f64 {
        self.rho
    }

    pub fn normalize(&self) -> Self {
        if self.rho == 1.0 {
            *self
        } else if self.rho == 0.0 {
            panic!("Tried to normalize zero vector.");
        } else {
            self.scale(1.0 / self.rho)
        }
    }

    pub fn shift(&self, dx: f64, dy: f64, dz: f64) -> Self {
        Self::new(self.x + dx, self.y + dy, self.z + dz)
    }

    pub fn turn(&self, dtheta: f64, dphi: f64) -> Self {
        Self::new_sph(self.rho, self.theta + dtheta, self.phi + dphi)
    }

    pub fn ons(&self) -> (Self, Self) {
        let v2 =
            if self.x.abs() > self.y.abs() {
                // project to the y = 0 plane and construct a normalized orthogonal vector in this plane
                let inv_len = 1.0 / (self.x.powi(2) + self.z.powi(2)).sqrt();
                Vector3::new(-self.z * inv_len, 0.0, self.x * inv_len)
            } else {
                // project to the x = 0 plane and construct a normalized orthogonal vector in this plane
                let inv_len = 1.0 / (self.y.powi(2) + self.z.powi(2)).sqrt();
                Vector3::new(0.0, self.z * inv_len, -self.y * inv_len)
            };
        let v3 = self.cross(v2);
        (v2, v3)
    }
}

impl Add for Vector3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Sub for Vector3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl Mul for Vector3 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }
}

impl Div for Vector3 {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self::new(self.x / other.x, self.y / other.y, self.z / other.z)
    }
}