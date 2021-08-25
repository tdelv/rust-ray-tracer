use crate::linalg::Vector3;

const EPS: f64 = 0.0001;

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub pos: Vector3, pub dir: Vector3
}

impl Ray {
    pub fn new(pos: Vector3, dir: Vector3) -> Self {
        Ray { pos, dir: dir.normalize() }
    }

    pub fn shift(&self, dx: f64, dy: f64, dz: f64) -> Self {
        Ray { pos: self.pos.shift(dx, dy, dz), dir: self.dir }
    }
    
    pub fn turn(&self, dtheta: f64, dphi: f64) -> Self {
        Ray { pos: self.pos, dir: self.dir.turn(dtheta, dphi) }
    }

    pub fn get_point(&self, t: f64) -> Vector3 {
        self.pos + self.dir.scale(t)
    }
}

pub trait Shape {
    fn intersect(&self, ray: Ray) -> Option<f64>;
    fn normal(&self, pos: Vector3) -> Vector3;
}

#[derive(Debug, Copy, Clone)]
pub struct Plane {
    pub point: Vector3, pub norm: Vector3
}

impl Plane {
    fn new(point: Vector3, norm: Vector3) -> Plane {
        Plane {
            point,
            norm: norm.normalize()
        }
    }
}

impl Shape for Plane {
    fn intersect(&self, ray: Ray) -> Option<f64> {
        let t = self.norm.dot(self.point - ray.pos) / self.norm.dot(ray.dir);
        if t > EPS {
            Some(t)
        } else {
            None
        }
    }

    fn normal(&self, _pos: Vector3) -> Vector3 {
        self.norm
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Sphere {
    pub center: Vector3, pub radius: f64
}

impl Shape for Sphere {
    fn intersect(&self, ray: Ray) -> Option<f64> {
        /*
            c=<cx, cy, cz>, r
            o=<ox, oy, oz>, d=<dx, dy, dz>

            p = o + d * t
            |p - c| = r

            (ox + t dx - cx)^2 + 
            (oy + t dy - cy)^2 + 
            (oz + t dz - cz)^2 = r^2

            (ox - cx)^2 + t^2 dx^2 + 2 t dx (ox - cx) + 
            (oy - cy)^2 + t^2 dy^2 + 2 t dy (oy - cy) + 
            (oz - cz)^2 + t^2 dz^2 + 2 t dz (oz - cz) = r^2

            t^2 (dx^2 + dy^2 + dz^2) +
            t 2 (dx (ox - cx) + dy (oy - cy) + dz (oz - dz)) +
            (ox - cx)^2 + (oy - cy)^2 + (oz - cz)^2 - r^2      = 0

            t^2 |d|^2 + t (2 * d.dot(o - c)) + (|o - c| - r^2) = 0

            a = |d|^2 = 1
            b = (2 * d.dot(o - c))
            c = (|o - c|^2 - r^2)

            t = (-b +/- sqrt(b^2 - 4ac)) / 2a
            = (-b +/- sqrt(b^2 - 4c)) / 2
        */
        let b = 2.0 * ray.dir.dot(ray.pos - self.center);
        let c = (ray.pos - self.center).size().powi(2) - self.radius.powi(2);
        let disc = b.powi(2) - 4.0 * c;
        if disc < 0.0 {
            None
        } else {
            let sqrtdisc = disc.sqrt();
            let t1 = -b + sqrtdisc;
            let t2 = -b - sqrtdisc;
            if t2 > EPS { 
                Some(t2 / 2.0) 
            } else if t1 > EPS { 
                Some(t1 / 2.0) 
            } else { 
                None 
            }
        }
    }

    fn normal(&self, pos: Vector3) -> Vector3 {
        (pos - self.center).scale(1.0 / self.radius)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Triangle {
    vertices: [Vector3; 3],
    plane: Plane
}

impl Triangle {
    pub fn new(v1: Vector3, v2: Vector3, v3: Vector3) -> Triangle {
        let norm = (v2 - v1).cross(v3 - v1);
        Triangle  {
            vertices: [v1, v2, v3],
            plane: Plane::new(v1, norm)
        }
    }

    pub fn vertices(&self) -> [Vector3; 3] {
        self.vertices
    }

    pub fn area(&self) -> f64 {
        let [v1, v2, v3] = self.vertices;

        let l1 = (v2 - v1).size();
        let l2 = (v3 - v1).size();
        let l3 = (v3 - v2).size();

        let p = (l1 + l2 + l3) / 2.0;
        let prod = p * (p - l1) * (p - l2) * (p - l3);
        prod.sqrt()
    }
}

impl Shape for Triangle {
    fn intersect(&self, ray: Ray) -> Option<f64> {
        self.plane
            .intersect(ray)
            .filter(|t| {
                let point = ray.get_point(*t);
                let [v1, v2, v3] = self.vertices;

                let tri1 = Triangle::new(v1, v2, point);
                let tri2 = Triangle::new(v1, v3, point);
                let tri3 = Triangle::new(v2, v3, point);

                (tri1.area() + tri2.area() + tri3.area() - self.area()).abs() < EPS
            })
    }

    fn normal(&self, pos: Vector3) -> Vector3 {
        self.plane.normal(pos)
    }
}