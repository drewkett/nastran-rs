use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Clone, Copy, Default)]
pub struct Vec3([f64; 3]);

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self([x, y, z])
    }

    pub fn dot(self, other: Vec3) -> f64 {
        let a = self.0;
        let b = other.0;
        a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
    }

    pub fn length(self) -> f64 {
        self.dot(self).sqrt()
    }

    pub fn normalize(self) -> Self {
        let l = self.length();
        self / l
    }

    pub fn cross(self, other: Self) -> Self {
        let a = self.0;
        let b = other.0;
        Self([
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ])
    }

    pub fn x(&self) -> f64 {
        self.0[0]
    }
    pub fn y(&self) -> f64 {
        self.0[1]
    }
    pub fn z(&self) -> f64 {
        self.0[2]
    }
}

impl Add<Self> for Vec3 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self([
            self.0[0] + other.0[0],
            self.0[1] + other.0[1],
            self.0[2] + other.0[2],
        ])
    }
}

impl std::ops::AddAssign<Vec3> for Vec3 {
    fn add_assign(&mut self, rhs: Vec3) {
        self.0[0] += rhs.0[0];
        self.0[1] += rhs.0[1];
        self.0[2] += rhs.0[2];
    }
}

impl Sub<Self> for Vec3 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self([
            self.0[0] - other.0[0],
            self.0[1] - other.0[1],
            self.0[2] - other.0[2],
        ])
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;
    fn mul(self, other: f64) -> Self {
        Self([self.0[0] * other, self.0[1] * other, self.0[2] * other])
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;
    fn mul(self, other: Vec3) -> Vec3 {
        Vec3([other.0[0] * self, other.0[1] * self, other.0[2] * self])
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;
    fn div(self, other: f64) -> Self {
        Self([self.0[0] / other, self.0[1] / other, self.0[2] / other])
    }
}

pub struct Mat3([[f64; 3]; 3]);

impl Mat3 {
    pub fn new(x: Vec3, y: Vec3, z: Vec3) -> Self {
        Self([x.0, y.0, z.0])
    }
}

impl Mul<Vec3> for &Mat3 {
    type Output = Vec3;
    fn mul(self, other: Vec3) -> Vec3 {
        let m = self.0;
        let v = other.0;
        Vec3([
            m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
            m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
            m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
        ])
    }
}

#[derive(Debug, Clone, Copy)]
pub struct XYZ(Vec3);

#[derive(Debug, Clone, Copy)]
pub struct DeltaXYZ(Vec3);

impl XYZ {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self(Vec3([x, y, z]))
    }
    pub fn x(&self) -> f64 {
        (self.0).0[0]
    }
    pub fn y(&self) -> f64 {
        (self.0).0[1]
    }
    pub fn z(&self) -> f64 {
        (self.0).0[2]
    }

    pub fn to_vec(self) -> Vec3 {
        self.0
    }
}

//impl std::ops::Deref for XYZ {
//    type Target = Vec3;
//
//    fn deref(&self) -> &Self::Target {
//        &self.0
//    }
//}

impl Sub<Self> for XYZ {
    type Output = DeltaXYZ;
    fn sub(self, other: Self) -> DeltaXYZ {
        DeltaXYZ(self.0 - other.0)
    }
}

//impl std::ops::Deref for DeltaXYZ {
//    type Target = Vec3;
//
//    fn deref(&self) -> &Self::Target {
//        &self.0
//    }
//}

pub struct CoordSys {
    m: Mat3,
    o: Vec3,
}

impl CoordSys {
    pub fn new(x: Vec3, y: Vec3, z: Vec3, o: Vec3) -> Self {
        CoordSys {
            m: Mat3::new(x, y, z),
            o,
        }
    }

    pub fn forward(&self, xyz: XYZ) -> XYZ {
        XYZ(&self.m * xyz.0 + self.o)
    }
}
