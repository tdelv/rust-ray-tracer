use crate::config::Config;
use crate::shapes::{Shape, Ray};
use crate::linalg::Vector3;

use rand::Rng;
use rayon::prelude::*;

pub type Color = Vector3;

impl Color {
    pub const BLACK: Color = Color { x: 0.0, y: 0.0, z: 0.0, rho: 0.0, theta: 0.0, phi: 0.0 };
    pub const WHITE: Color = Color { x: 255.0, y: 255.0, z: 255.0, rho: 0.0, theta: 0.0, phi: 0.0 };
    pub const RED: Color = Color { x: 255.0, y: 0.0, z: 0.0, rho: 0.0, theta: 0.0, phi: 0.0 };
    pub const GREEN: Color = Color { x: 0.0, y: 255.0, z: 0.0, rho: 0.0, theta: 0.0, phi: 0.0 };
    pub const BLUE: Color = Color { x: 0.0, y: 0.0, z: 255.0, rho: 0.0, theta: 0.0, phi: 0.0 };
    pub const YELLOW: Color = Color { x: 255.0, y: 255.0, z: 0.0, rho: 0.0, theta: 0.0, phi: 0.0 };

    pub fn from_string(s: &str) -> Option<Color> {
        match s {
            "black" => Some(Color::BLACK),
            "white" => Some(Color::WHITE),
            "red" => Some(Color::RED),
            "green" => Some(Color::GREEN),
            "blue" => Some(Color::BLUE),
            "yellow" => Some(Color::YELLOW),
            _ => None
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Material {
    Mirror,
    Translucent(f64),
}

pub struct Object {
    pub shape: Box<dyn Shape>, 
    pub color: Color, 
    pub lum: Color,
    pub material: Material
}
unsafe impl Sync for Object {}

fn get_color(objects: &[Object], ray: Ray, depth: u16) -> Color {
    if depth == 0 {
        Color::BLACK
    } else {
        let obj_ts = objects.iter()
            .filter_map(|obj| obj.shape.intersect(ray).map(|t| (obj, t)))
            .reduce(|(o1, t1), (o2, t2)| if t1 < t2 { (o1, t1) } else { (o2, t2) });
        match obj_ts {
            None => Color::BLACK,
            Some((best_obj, best_t)) => {
                let new_pos = ray.pos + ray.dir.scale(best_t);

                let n = best_obj.shape.normal(new_pos);
                let cost = ray.dir.dot(n);

                let reflected = match &best_obj.material {
                    Material::Mirror => {
                        let new_dir = ray.dir - n.scale(2.0 * cost);
                        let new_ray = Ray { pos: new_pos, dir: new_dir };
                        let incoming = get_color(objects, new_ray, depth - 1);
                        (incoming * best_obj.color).scale(1.0/255.0)
                    },
                    Material::Translucent(clearness) => {
                        let rand: f64 = rand::random();
                        if rand < *clearness { // Glass
                            // let new_dir = ray.dir - n.scale(2.0 * cost);
                            // let new_ray = Ray { pos: new_pos, dir: new_dir };
                            // let incoming = get_color(objects, new_ray, depth - 1);
                            // incoming * best_obj.color
                            let refr: f64 = 1.5;
                            let r0: f64 = (1.0 - refr) / (1.0 + refr);
                            let r0 = r0 * r0;
                            let (n, refr) =
                                if n.dot(ray.dir) > 0.0 { // we're inside the medium
                                    (n.scale(-1.0), refr)
                                } else {
                                    (n, 1.0 / refr)
                                };
                            let cost1: f64 = n.dot(ray.dir) * -1.0; // cosine of theta_1
                            let cost2: f64 = 1.0 - refr.powi(2) * (1.0 - cost1.powi(2)); // cosine of theta_2
                            let r_prob: f64 = r0 + (1.0 - r0) * (1.0 - cost1).powi(5); // Schlick-approximation
                            let new_dir = 
                                if cost2 > 0.0 && rand::thread_rng().gen::<f64>() > r_prob { // refraction direction
                                    (ray.dir.scale(refr) + n.scale(refr * cost1 - cost2.sqrt())).normalize()
                                } else { // reflection direction
                                    (ray.dir + n.scale(cost1 * 2.0)).normalize()
                                };
                            let new_ray = Ray::new(new_pos, new_dir);

                            let incoming = get_color(objects, new_ray, depth - 1);
                            incoming.scale(1.15).scale(1.0 / 0.9)
                        } else { // Opaque
                            let n = if cost < 0.0 { n } else { n.scale(-1.0) };
                            let cost = cost.abs();

                            let (rot_x, rot_y) = n.ons();
                            let sampled_dir = Vector3::rand_hemi2();
                            let new_dir = Vector3::new(
                                Vector3::new(rot_x.x, rot_y.x, n.x).dot(sampled_dir),
                                Vector3::new(rot_x.y, rot_y.y, n.y).dot(sampled_dir),
                                Vector3::new(rot_x.z, rot_y.z, n.z).dot(sampled_dir)
                            );
                            let new_ray = Ray::new(new_pos, new_dir);

                            let incoming = get_color(objects, new_ray, depth - 1);
                            let cost = new_dir.dot(n);
                            (incoming * best_obj.color).scale(cost).scale(1.0/255.0).scale(1.0/0.9)
                        }
                    }
                };

                reflected + best_obj.lum
                // best_obj.color
            }
        }
    }
}

pub fn make_image(config: &Config) -> Vec<Vec<Vector3>> {
    (0..config.height).into_par_iter().map(|y| {
        (0..config.width).into_par_iter().map(|x| {
            let mut rng = rand::thread_rng();
            
            let xf = x as f64;
            let yf = (config.height - y - 1) as f64;

            let widthf = config.width as f64;
            let heightf = config.height as f64;

            let fovx = config.fov;
            let fovy = fovx * (heightf / widthf);

            let dtheta = - ((2.0 * xf - widthf) / widthf) * fovx;
            let dphi = - ((2.0 * yf - heightf) / heightf) * fovy;
            let ray = config.pov.turn(dtheta, dphi);

            let mut r = 0.0;
            let mut g = 0.0;
            let mut b = 0.0;
            for _ in 0..config.num_tries {
                let ray = ray.turn(
                    (2.0 * rng.gen::<f64>() - 1.0) * config.max_variation, 
                    (2.0 * rng.gen::<f64>() - 1.0) * config.max_variation);
                let color = get_color(&config.objects, ray, config.max_depth);
                r += color.x;
                g += color.y;
                b += color.z;
            }

            Vector3::new(r, g, b)
        }).collect()
    }).collect()
}
