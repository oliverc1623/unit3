pub use cgmath::prelude::*;
pub type Vec3 = cgmath::Vector3<f32>;
pub type Pos3 = cgmath::Point3<f32>;
pub type Vec4 = cgmath::Vector4<f32>;
pub type Mat3 = cgmath::Matrix3<f32>;
pub type Mat4 = cgmath::Matrix4<f32>;
pub type Quat = cgmath::Quaternion<f32>;
pub const PI: f32 = std::f32::consts::PI;
use std::num;

pub trait Shape {
    fn translate(&mut self, v: Vec3);
    fn apply_impulse(&mut self, v: Vec3);
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Sphere {
    pub c: Pos3,
    pub r: f32,
    pub lin_mom: Vec3,
    pub ang_mom: Vec3,
    pub mass: f32,
}

impl Shape for Sphere {
    fn translate(&mut self, v: Vec3) {
        self.c += v;
    }
    fn apply_impulse(&mut self, disp: Vec3) {

        // This is calculating and applying the impulse, the assumption is that it is only stationaty contacts
        // let bounce = 0.9;
        // let d = self.lin_mom.dot(n);
        // let j = (-(1.0 + bounce) * d).max(0.0);

        let mr = (2.0/5.0) * self.mass * self.r * self.r;
        let contact_point = self.c.to_vec() + disp.normalize() * self.r; // The point of contact


        let m = self.mass;
        let e = 0.9;
        let n = disp / disp.magnitude();
        let v = self.lin_mom + self.ang_mom.cross(contact_point - self.c.to_vec());
        let r = contact_point - self.c.to_vec();
        let i = Mat3::new(mr, 0.0, 0.0, 0.0, mr, 0.0, 0.0, 0.0, mr);
        let u = 0.5;
        let t = -v;



        let num = -(1.0 + e) * v.dot(n);
        let den = (1.0 / m) + (i.invert().unwrap() * (r.cross(n)).cross(r)).dot(n);
        // let num = (-v).dot(t) * u;
        // let den = (1.0 / m) + (i.invert().unwrap() * (r.cross(t)).cross(r)).dot(t);
        println!("den: {} ", den);
        let j_new = num / den;

        self.lin_mom += j_new * n; // Update linear momentum
        self.ang_mom += r.cross(j_new * n); // Update angular momentum
        println!("{},{},{}", self.ang_mom.x, self.ang_mom.y, self.ang_mom.z);
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Plane {
    pub n: Vec3,
    pub d: f32,
}

impl Shape for Plane {
    fn translate(&mut self, _v: Vec3) {
        panic!();
    }
    fn apply_impulse(&mut self, _v: Vec3) {
        panic!();
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Box {
    pub c: Pos3,
    pub axes: Mat3,
    pub half_sizes: Vec3,
}

impl Shape for Box {
    fn translate(&mut self, v: Vec3) {
        self.c += v;
    }
    fn apply_impulse(&mut self, _v: Vec3) {
        panic!();
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct AABB {
    pub c: Pos3,
    pub half_sizes: Vec3,
}

impl Shape for AABB {
    fn translate(&mut self, v: Vec3) {
        self.c += v;
    }
    fn apply_impulse(&mut self, _v: Vec3) {
        panic!();
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Ray {
    pub p: Pos3,
    pub dir: Vec3,
}

impl Shape for Ray {
    fn translate(&mut self, v: Vec3) {
        self.p += v;
    }
    fn apply_impulse(&mut self, _v: Vec3) {
        panic!();
    }
}

pub trait Collide<S: Shape>: Shape {
    fn touching(&self, s2: &S) -> bool {
        self.disp(s2).is_some()
    }
    fn disp(&self, s2: &S) -> Option<Vec3>;
}

impl Collide<Sphere> for Sphere {
    fn touching(&self, s2: &Sphere) -> bool {
        // Is the (squared) distance between the centers less than the
        // (squared) sum of the radii?
        s2.c.distance2(self.c) <= (self.r + s2.r).powi(2)
    }
    /// What's the offset I'd need to push s1 and s2 out of each other?
    fn disp(&self, s2: &Sphere) -> Option<Vec3> {
        let offset = s2.c - self.c;
        let distance = offset.magnitude();
        if distance < self.r + s2.r {
            // Make sure we don't divide by 0
            let distance = if distance == 0.0 { 1.0 } else { distance };
            // How much combined radius is "left over"?
            let disp_mag = (self.r + s2.r) - distance;
            // Normalize offset and multiply by the amount to push
            Some(offset * (disp_mag / distance))
        } else {
            None
        }
    }
}

impl Collide<Plane> for Sphere {
    fn touching(&self, p: &Plane) -> bool {
        // Find the distance of the sphere's center to the plane
        (self.c.dot(p.n) - p.d).abs() <= self.r
    }
    fn disp(&self, p: &Plane) -> Option<Vec3> {
        // Find the distance of the sphere's center to the plane
        let dist = self.c.dot(p.n) - p.d;
        if dist.abs() <= self.r {
            // If we offset from the sphere position opposite the normal,
            // we'll end up hitting the plane at `dist` units away.  So
            // the displacement is just the plane's normal * dist.
            Some(p.n * (self.r - dist))
        } else {
            None
        }
    }
}

impl Collide<AABB> for Sphere {
    fn touching(&self, b: &AABB) -> bool {
        let minX = b.c.x - b.half_sizes[0];
        let maxX = b.c.x + b.half_sizes[0];
        let minY = b.c.y - b.half_sizes[1];
        let maxY = b.c.y + b.half_sizes[1];
        let minZ = b.c.z - b.half_sizes[2];
        let maxZ = b.c.z + b.half_sizes[2];

        let x = minX.max(self.c.x.min(maxX));
        let y = minY.max(self.c.y.min(maxY));
        let z = minZ.max(self.c.z.min(maxZ));

        let vals = Vec3::new(x, y, z);
        let distance = vals.distance(self.c.to_vec());
        if distance < self.r{
            println!("collide! {}, {}", distance, self.r);
        }
        return distance < self.r;
    }

    fn disp(&self, b: &AABB) -> Option<Vec3>{
        let minX = b.c.x - b.half_sizes[0] * 2.0;
        let maxX = b.c.x + b.half_sizes[0] * 2.0;
        let minY = b.c.y - b.half_sizes[1] * 2.0;
        let maxY = b.c.y + b.half_sizes[1] * 2.0;
        let minZ = b.c.z - b.half_sizes[2] * 2.0;
        let maxZ = b.c.z + b.half_sizes[2] * 2.0;

        let x = minX.max(self.c.x.min(maxX));
        let y = minY.max(self.c.y.min(maxY));
        let z = minZ.max(self.c.z.min(maxZ));

        let cp = Vec3::new(x, y, z);
        let distance = cp.distance(self.c.to_vec());
        if distance < self.r {
            println!("collide! {}, {}", distance, self.r);
            // return Some(cp - self.c.to_vec());
            return None;
        }
        None
    }
}

type CastHit = Option<(Pos3, f32)>;

trait Cast<S: Shape> {
    fn cast(&self, s: &S) -> CastHit;
}

impl Cast<Sphere> for Ray {
    fn cast(&self, s: &Sphere) -> CastHit {
        let m = self.p - s.c;
        let b = self.dir.dot(m);
        let c = m.dot(m) - s.r * s.r;
        let discr = b * b - c;
        if (c > 0.0 && b > 0.0) || discr < 0.0 {
            return None;
        }
        let t = (-b - discr.sqrt()).max(0.0);
        Some((self.p + t * self.dir, t))
    }
}
impl Cast<Plane> for Ray {
    fn cast(&self, b: &Plane) -> CastHit {
        let denom = self.dir.dot(b.n);
        if denom == 0.0 {
            return None;
        }
        let t = (b.d - self.p.dot(b.n)) / denom;
        if t >= 0.0 {
            Some((self.p + self.dir * t, t))
        } else {
            None
        }
    }
}
impl Cast<Box> for Ray {
    fn cast(&self, b: &Box) -> CastHit {
        let mut tmin = 0.0_f32;
        let mut tmax = f32::MAX;
        let delta = b.c - self.p;
        for i in 0..3 {
            let axis = b.axes[i];
            let e = axis.dot(delta);
            let mut f = self.dir.dot(axis);
            if f.abs() < f32::EPSILON {
                if -e - b.half_sizes[i] > 0.0 || -e + b.half_sizes[i] < 0.0 {
                    return None;
                }
                f = f32::EPSILON;
            }
            let mut t1 = (e + b.half_sizes[i]) / f;
            let mut t2 = (e - b.half_sizes[i]) / f;
            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }
            tmin = tmin.max(t1);
            tmax = tmax.min(t2);
            if tmin > tmax {
                return None;
            }
        }
        Some((self.p + self.dir * tmin, tmin))
    }
}
impl Cast<AABB> for Ray {
    fn cast(&self, b: &AABB) -> CastHit {
        let mut tmin = 0.0_f32;
        let mut tmax = f32::MAX;
        let min = b.c - b.half_sizes;
        let max = b.c + b.half_sizes;
        for i in 0..3 {
            if self.dir[i].abs() < f32::EPSILON {
                if self.p[i] < min[i] {
                    return None;
                }
                continue;
            }
            let ood = 1.0 / self.dir[i];
            let mut t1 = (min[i] - self.p[i]) * ood;
            let mut t2 = (max[i] - self.p[i]) * ood;
            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }
            tmin = tmin.max(t1);
            tmax = tmax.min(t2);
            if tmin > tmax {
                return None;
            }
        }
        Some((self.p + self.dir * tmin, tmin))
    }
}

struct Bivector3 {
    xy: f32,
    xz: f32,
    yz: f32
}

impl Bivector3 {

    fn new(xy: f32, xz: f32, yz: f32) -> Self {
        Bivector3 {
            xy,
            xz,
            yz
        }
    }

}


// struct Rotor {

// }
