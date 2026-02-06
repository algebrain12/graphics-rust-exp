
use std::{num::NonZeroI16, ops::{Add, Mul, Sub}};

use {
    bytemuck::{Pod, Zeroable},
    std::ops,
};

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct Vec4([f32; 4]);

impl Default for Vec4 {
    fn default() -> Self {
        Self::zero()
    }
}

macro_rules! impl_binary_op {
    ($op:tt : $method:ident => (
           $lhs_i:ident : $lhs_t:path,
           $rhs_i:ident : $rhs_t:path
        ) -> $return_t:path $body:block
    ) => {
        impl ops::$op<$rhs_t> for $lhs_t {
            type Output = $return_t;
            fn $method(self, $rhs_i: $rhs_t) -> $return_t {
                let $lhs_i = self;
                $body
            }
        }
        impl ops::$op<&$rhs_t> for $lhs_t {
            type Output = $return_t;
            fn $method(self, $rhs_i: &$rhs_t) -> $return_t {
                let $lhs_i = self;
                $body
            }
        }
        impl ops::$op<$rhs_t> for &$lhs_t {
            type Output = $return_t;
            fn $method(self, $rhs_i: $rhs_t) -> $return_t {
                let $lhs_i = self;
                $body
            }
        }
        impl ops::$op<&$rhs_t> for &$lhs_t {
            type Output = $return_t;
            fn $method(self, $rhs_i: &$rhs_t) -> $return_t {
                let $lhs_i = self;
                $body
            }
        }
    };
}

impl_binary_op!(Add : add => (lhs: Vec4, rhs: Vec4) -> Vec4 {
    Vec4([
        lhs.x() + rhs.x(),
        lhs.y() + rhs.y(),
        lhs.z() + rhs.z(),
        lhs.w() + rhs.w(),
    ])
});

impl_binary_op!(Sub : sub => (lhs: Vec4, rhs: Vec4) -> Vec4 {
    Vec4([
        lhs.x() - rhs.x(),
        lhs.y() - rhs.y(),
        lhs.z() - rhs.z(),
        lhs.w() - rhs.w(),
    ])
});

impl_binary_op!(Mul : mul => (lhs: Vec4, rhs: f32) -> Vec4 {
    Vec4([
        lhs.x() * rhs,
        lhs.y() * rhs,
        lhs.z() * rhs,
        lhs.w() * rhs,
    ])
});

impl_binary_op!(Mul : mul => (lhs: f32, rhs: Vec4) -> Vec4 {
    Vec4([
        rhs.x() * lhs,
        rhs.y() * lhs,
        rhs.z() * lhs,
        rhs.w() * lhs,
    ])
});

impl_binary_op!(Div : div => (lhs: Vec4, rhs: f32) -> Vec4 {
    Vec4([
        lhs.x() / rhs,
        lhs.y() / rhs,
        lhs.z() / rhs,
        lhs.w() / rhs,
    ])
});

impl ops::AddAssign for Vec4 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl ops::SubAssign for Vec4 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl ops::MulAssign<f32> for Vec4 {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl ops::DivAssign<f32> for Vec4 {
    fn div_assign(&mut self, rhs: f32) {
        *self = *self / rhs;
    }
}

impl ops::Neg for Vec4 {
    type Output = Vec4;
    fn neg(self) -> Self::Output {
        Vec4([
            -self.x(),
            -self.y(),
            -self.z(),
            -self.w(),
        ])
    }
}

impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Vec4 {
        Vec4([x, y, z, w])
    }

    pub fn all(v: f32) -> Vec4 {
        Vec4([v, v, v, v])
    }

    pub fn zero() -> Vec4 {
        Vec4([0., 0., 0., 0.])
    }

    #[inline(always)]
    pub fn x(&self) -> f32 { self.0[0] }
    #[inline(always)]
    pub fn y(&self) -> f32 { self.0[1] }
    #[inline(always)]
    pub fn z(&self) -> f32 { self.0[2] }
    #[inline(always)]
    pub fn w(&self) -> f32 { self.0[3] }

    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn length_squared(&self) -> f32 {
        self.dot(self)
    }

    pub fn dot(&self, rhs: &Vec4) -> f32 {
        self.x() * rhs.x() +
        self.y() * rhs.y() +
        self.z() * rhs.z() +
        self.w() * rhs.w()
    }
    pub fn cross(&self, rhs: &Vec4) -> Vec4 {
    Vec4([
        self.y() * rhs.z() - self.z() * rhs.y(),
        self.z() * rhs.x() - self.x() * rhs.z(),
        self.x() * rhs.y() - self.y() * rhs.x(),
        self.w()
    ])
    }
    pub fn normalized(self) -> Vec4 {
        self * self.length().recip()
    }
}




#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct CameraUniforms {
    origin: Vec4,
    u: Vec4,
    v: Vec4,
    w: Vec4,
}

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct Camera {
    uniforms: CameraUniforms,
    pitch: f32,
    yaw: f32,
}

impl Camera {
    pub fn new(origin: Vec4) -> Camera {
        Camera {
            uniforms: CameraUniforms { 
                origin, 
                u: origin,
                v: origin,
                w: origin,
            },
            pitch:0.0,
            yaw:0.0,
        }
    }

    pub fn uniforms(&self) -> &CameraUniforms {
        &self.uniforms
    }

    pub fn look_at(origin: Vec4, center: Vec4, up: Vec4) -> Camera {
        let w = (center - origin).normalized();
        let u = w.cross(&up).normalized();
        let v = u.cross(&w);
        Camera {
            uniforms: CameraUniforms {
                origin: origin,
                u: u,
                v: v,
                w: w,
            },
            pitch:0.0,
            yaw:0.0
        }
    }
    pub fn zoom(&mut self, displacement: f32) {
        self.uniforms.origin += displacement * self.uniforms.w;
    }
    pub fn moves(&mut self, x:f32, y:f32, z:f32) {
        self.uniforms.w += Vec4::new(x, y, z, 0.0);
    }
    pub fn rotate(&mut self, pitch:f32, yaw:f32){
        self.pitch+=pitch;
        self.yaw+=yaw;
        let ninety = 90.0/180.0*3.1415926;
        if(self.pitch > ninety){
            self.pitch = ninety;
        }
        if(self.pitch < -ninety){
            self.pitch = -ninety;
        }
    }
    pub fn setrotation(&mut self, pitch:f32, yaw:f32){
        self.pitch=pitch;
        self.yaw=yaw;
        let ninety = 89.0/180.0*3.1415926;
        if(self.yaw > ninety){
            self.yaw = ninety;
        }
        if(self.yaw < -ninety){
            self.yaw = -ninety;
        }
    }
    pub fn set_w(&mut self){
        self.uniforms.w = Vec4::new((self.pitch).cos()*(self.yaw).sin(),-(self.pitch).sin(),-(self.pitch).cos()*(self.yaw.cos()),0.0);
    }
    pub fn moveset(&mut self, x:f32, y:f32, z:f32) {
        self.uniforms.w = Vec4::new(x, y, z, 0.0);
    }
    pub fn shift(&mut self, x:f32, y:f32, z:f32){
        let fx = x*self.yaw.cos()-z*self.yaw.sin();
        let fz = x*self.yaw.sin()+z*self.yaw.cos();
        self.uniforms.origin += Vec4::new(fx, y, fz, 0.0);
    }

}



