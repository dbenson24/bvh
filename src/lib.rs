//! A crate which exports rays, axis-aligned bounding boxes, and binary bounding
//! volume hierarchies.
//!
//! ## About
//!
//! This crate can be used for applications which contain intersection computations of rays
//! with primitives. For this purpose a binary tree BVH (Bounding Volume Hierarchy) is of great
//! use if the scene which the ray traverses contains a huge number of primitives. With a BVH the
//! intersection test complexity is reduced from O(n) to O(log2(n)) at the cost of building
//! the BVH once in advance. This technique is especially useful in ray/path tracers. For
//! use in a shader this module also exports a flattening procedure, which allows for
//! iterative traversal of the BVH.
//!
//! ## Example
//!
//! ```
//! use bvh::aabb::{AABB, Bounded};
//! use bvh::bounding_hierarchy::{BoundingHierarchy, BHShape};
//! use bvh::bvh::BVH;
//! use bvh::{Point3, Vector3};
//! use bvh::ray::Ray;
//! use bvh::Real;
//!
//! let origin = Point3::new(0.0,0.0,0.0);
//! let direction = Vector3::new(1.0,0.0,0.0);
//! let ray = Ray::new(origin, direction);
//!
//! struct Sphere {
//!     position: Point3,
//!     radius: Real,
//!     node_index: usize,
//! }
//!
//! impl Bounded for Sphere {
//!     fn aabb(&self) -> AABB {
//!         let half_size = Vector3::new(self.radius, self.radius, self.radius);
//!         let min = self.position - half_size;
//!         let max = self.position + half_size;
//!         AABB::with_bounds(min, max)
//!     }
//! }
//!
//! impl BHShape for Sphere {
//!     fn set_bh_node_index(&mut self, index: usize) {
//!         self.node_index = index;
//!     }
//!
//!     fn bh_node_index(&self) -> usize {
//!         self.node_index
//!     }
//! }
//!
//! let mut spheres = Vec::new();
//! for i in 0..1000u32 {
//!     let position = Point3::new(i as Real, i as Real, i as Real);
//!     let radius = (i % 10) as Real + 1.0;
//!     spheres.push(Sphere {
//!         position: position,
//!         radius: radius,
//!         node_index: 0,
//!     });
//! }
//!
//! let bvh = BVH::build(&mut spheres);
//! let hit_sphere_aabbs = bvh.traverse(&ray, &spheres);
//! ```
//!
//! ## Features
//!
//! - `serde_impls` (default **disabled**) - adds `Serialize` and `Deserialize` implementations for some types
//!

#![deny(missing_docs)]
#![cfg_attr(feature = "bench", feature(test))]

#[cfg(all(feature = "bench", test))]
extern crate test;

/// Point math type used by this crate. Type alias for [`glam::DVec3`].
#[cfg(feature = "f64")]
pub type Point3 = glam::DVec3;

/// Vector math type used by this crate. Type alias for [`glam::DVec3`].
#[cfg(feature = "f64")]
pub type Vector3 = glam::DVec3;

/// Matrix math type used by this crate. Type alias for [`glam::DMat4`].
#[cfg(feature = "f64")]
pub type Mat4 = glam::DMat4;

/// Matrix math type used by this crate. Type alias for [`glam::DQuat`].
#[cfg(feature = "f64")]
pub type Quat = glam::DQuat;

#[cfg(feature = "f64")]
/// Float type used by this crate
pub type Real = f64;

/// Point math type used by this crate. Type alias for [`glam::Vec3`].
#[cfg(not(feature = "f64"))]
pub type Point3 = glam::Vec3;

/// Vector math type used by this crate. Type alias for [`glam::Vec3`].
#[cfg(not(feature = "f64"))]
pub type Vector3 = glam::Vec3;

/// Matrix math type used by this crate. Type alias for [`glam::Mat4`].
#[cfg(not(feature = "f64"))]
pub type Mat4 = glam::Mat4;

/// Quat math type used by this crate. Type alias for [`glam::Quat`].
#[cfg(not(feature = "f64"))]
pub type Quat = glam::Quat;

#[cfg(not(feature = "f64"))]
/// Float type used by this crate
pub type Real = f32;

/// A minimal floating value used as a lower bound.
/// TODO: replace by/add ULPS/relative float comparison methods.
pub const EPSILON: Real = 0.00001;

pub mod axis;
pub mod bounding_hierarchy;
pub mod bvh;
pub mod flat_bvh;
mod shapes;
mod utils;

#[cfg(test)]
mod testbase;

pub use shapes::*;
//pub use shapes::{Ray, AABB, OBB, Capsule, Sphere};
use aabb::{Bounded, AABB};
use bounding_hierarchy::BHShape;


#[derive(Debug)]
struct Sphere {
    position: Point3,
    radius: Real,
    node_index: usize,
}

impl Bounded for Sphere {
    fn aabb(&self) -> AABB {
        let half_size = Vector3::new(self.radius, self.radius, self.radius);
        let min = self.position - half_size;
        let max = self.position + half_size;
        AABB::with_bounds(min, max)
    }
}

impl BHShape for Sphere {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}

/// A triangle struct. Instance of a more complex `Bounded` primitive.
#[derive(Debug)]
pub struct Triangle {
    /// First point on the triangle
    pub a: Point3,
    /// Second point on the triangle
    pub b: Point3,
    /// Third point on the triangle
    pub c: Point3,
    aabb: AABB,
    node_index: usize,
}

impl Triangle {
    /// Creates a new triangle given a clockwise set of points
    pub fn new(a: Point3, b: Point3, c: Point3) -> Triangle {
        Triangle {
            a,
            b,
            c,
            aabb: AABB::empty().grow(&a).grow(&b).grow(&c),
            node_index: 0,
        }
    }
}

impl Bounded for Triangle {
    fn aabb(&self) -> AABB {
        self.aabb
    }
}
