//! This module defines a Ray structure and intersection algorithms
//! for axis aligned bounding boxes and triangles.

use crate::aabb::AABB;
use crate::bounding_hierarchy::IntersectionAABB;
use crate::{Point3, Vector3};
use crate::{Real, EPSILON};

/// A struct which defines a ray and some of its cached values.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde_impls", derive(serde::Serialize, serde::Deserialize))]
pub struct Ray {
    /// The ray origin.
    pub origin: Point3,

    /// The ray direction.
    pub direction: Vector3,

    /// Inverse (1/x) ray direction. Cached for use in [`AABB`] intersections.
    ///
    /// [`AABB`]: struct.AABB.html
    ///
    inv_direction: Vector3,

    /// Sign of the X direction. 0 means positive, 1 means negative.
    /// Cached for use in [`AABB`] intersections.
    ///
    /// [`AABB`]: struct.AABB.html
    ///
    sign_x: usize,

    /// Sign of the Y direction. 0 means positive, 1 means negative.
    /// Cached for use in [`AABB`] intersections.
    ///
    /// [`AABB`]: struct.AABB.html
    ///
    sign_y: usize,

    /// Sign of the Z direction. 0 means positive, 1 means negative.
    /// Cached for use in [`AABB`] intersections.
    ///
    /// [`AABB`]: struct.AABB.html
    ///
    sign_z: usize,
}

/// A struct which is returned by the `intersects_triangle` method.
#[derive(Debug, Clone, Copy)]
pub struct Intersection {
    /// Distance from the ray origin to the intersection point.
    pub distance: Real,

    /// U coordinate of the intersection.
    pub u: Real,

    /// V coordinate of the intersection.
    pub v: Real,

    /// Normal of the intersection
    pub norm: Vector3,

    /// Whether the intersect was a backface
    pub back_face: bool,
}

impl Intersection {
    /// Constructs an `Intersection`. `distance` should be set to positive infinity,
    /// if the intersection does not occur.
    pub fn new(distance: Real, u: Real, v: Real, norm: Vector3, back_face: bool) -> Intersection {
        Intersection {
            distance,
            u,
            v,
            norm,
            back_face,
        }
    }
}

/// This trait can be implemented on anything that can intersect with a `Ray`
pub trait IntersectionRay {
    /// Returns true if there is an intersection with the given `Ray`
    fn intersects_ray(&self, ray: &Ray, t_min: Real, t_max: Real) -> Option<Intersection>;
}

impl<T: ?Sized> IntersectionRay for &T
where
    T: IntersectionRay,
{
    fn intersects_ray(&self, ray: &Ray, t_min: Real, t_max: Real) -> Option<Intersection> {
        (*self).intersects_ray(ray, t_min, t_max)
    }
}

impl IntersectionAABB for Ray {
    /// Tests the intersection of a [`Ray`] with an [`AABB`] using the optimized algorithm
    /// from [this paper](http://www.cs.utah.edu/~awilliam/box/box.pdf).
    ///
    /// # Examples
    /// ```
    /// use bvh::aabb::AABB;
    /// use bvh::ray::Ray;
    /// use bvh::{Point3,Vector3};
    /// use bvh::bounding_hierarchy::IntersectionAABB;
    ///
    /// let origin = Point3::new(0.0,0.0,0.0);
    /// let direction = Vector3::new(1.0,0.0,0.0);
    /// let ray = Ray::new(origin, direction);
    ///
    /// let point1 = Point3::new(99.9,-1.0,-1.0);
    /// let point2 = Point3::new(100.1,1.0,1.0);
    /// let aabb = AABB::with_bounds(point1, point2);
    ///
    /// assert!(ray.intersects_aabb(&aabb));
    /// ```
    ///
    /// [`Ray`]: struct.Ray.html
    /// [`AABB`]: struct.AABB.html
    ///
    fn intersects_aabb(&self, aabb: &AABB) -> bool {
        let mut ray_min = (aabb[self.sign_x].x - self.origin.x) * self.inv_direction.x;
        let mut ray_max = (aabb[1 - self.sign_x].x - self.origin.x) * self.inv_direction.x;

        let y_min = (aabb[self.sign_y].y - self.origin.y) * self.inv_direction.y;
        let y_max = (aabb[1 - self.sign_y].y - self.origin.y) * self.inv_direction.y;

        if (ray_min > y_max) || (y_min > ray_max) {
            return false;
        }

        if y_min > ray_min {
            ray_min = y_min;
        }
        // Using the following solution significantly decreases the performance
        // ray_min = ray_min.max(y_min);

        if y_max < ray_max {
            ray_max = y_max;
        }
        // Using the following solution significantly decreases the performance
        // ray_max = ray_max.min(y_max);

        let z_min = (aabb[self.sign_z].z - self.origin.z) * self.inv_direction.z;
        let z_max = (aabb[1 - self.sign_z].z - self.origin.z) * self.inv_direction.z;

        if (ray_min > z_max) || (z_min > ray_max) {
            return false;
        }

        // Only required for bounded intersection intervals.
        // if z_min > ray_min {
        // ray_min = z_min;
        // }

        if z_max < ray_max {
            ray_max = z_max;
        }
        // Using the following solution significantly decreases the performance
        // ray_max = ray_max.min(y_max);

        ray_max > 0.0
    }
}

impl Ray {
    /// Creates a new [`Ray`] from an `origin` and a `direction`.
    /// `direction` will be normalized.
    ///
    /// # Examples
    /// ```
    /// use bvh::ray::Ray;
    /// use bvh::{Point3,Vector3};
    ///
    /// let origin = Point3::new(0.0,0.0,0.0);
    /// let direction = Vector3::new(1.0,0.0,0.0);
    /// let ray = Ray::new(origin, direction);
    ///
    /// assert_eq!(ray.origin, origin);
    /// assert_eq!(ray.direction, direction);
    /// ```
    ///
    /// [`Ray`]: struct.Ray.html
    ///
    pub fn new(origin: Point3, direction: Vector3) -> Ray {
        let direction = direction.normalize();
        Ray {
            origin,
            direction,
            inv_direction: Vector3::new(1.0 / direction.x, 1.0 / direction.y, 1.0 / direction.z),
            sign_x: (direction.x < 0.0) as usize,
            sign_y: (direction.y < 0.0) as usize,
            sign_z: (direction.z < 0.0) as usize,
        }
    }

    /// Naive implementation of a [`Ray`]/[`AABB`] intersection algorithm.
    ///
    /// # Examples
    /// ```
    /// use bvh::aabb::AABB;
    /// use bvh::ray::Ray;
    /// use bvh::{Point3,Vector3};
    ///
    /// let origin = Point3::new(0.0,0.0,0.0);
    /// let direction = Vector3::new(1.0,0.0,0.0);
    /// let ray = Ray::new(origin, direction);
    ///
    /// let point1 = Point3::new(99.9,-1.0,-1.0);
    /// let point2 = Point3::new(100.1,1.0,1.0);
    /// let aabb = AABB::with_bounds(point1, point2);
    ///
    /// assert!(ray.intersects_aabb_naive(&aabb));
    /// ```
    ///
    /// [`Ray`]: struct.Ray.html
    /// [`AABB`]: struct.AABB.html
    ///
    pub fn intersects_aabb_naive(&self, aabb: &AABB) -> bool {
        let hit_min_x = (aabb.min.x - self.origin.x) * self.inv_direction.x;
        let hit_max_x = (aabb.max.x - self.origin.x) * self.inv_direction.x;

        let hit_min_y = (aabb.min.y - self.origin.y) * self.inv_direction.y;
        let hit_max_y = (aabb.max.y - self.origin.y) * self.inv_direction.y;

        let hit_min_z = (aabb.min.z - self.origin.z) * self.inv_direction.z;
        let hit_max_z = (aabb.max.z - self.origin.z) * self.inv_direction.z;

        let x_entry = hit_min_x.min(hit_max_x);
        let y_entry = hit_min_y.min(hit_max_y);
        let z_entry = hit_min_z.min(hit_max_z);
        let x_exit = hit_min_x.max(hit_max_x);
        let y_exit = hit_min_y.max(hit_max_y);
        let z_exit = hit_min_z.max(hit_max_z);

        let latest_entry = x_entry.max(y_entry).max(z_entry);
        let earliest_exit = x_exit.min(y_exit).min(z_exit);

        latest_entry < earliest_exit && earliest_exit > 0.0
    }

    /// Implementation of the algorithm described [here]
    /// (https://tavianator.com/fast-branchless-raybounding-box-intersections/).
    ///
    /// # Examples
    /// ```
    /// use bvh::aabb::AABB;
    /// use bvh::ray::Ray;
    /// use bvh::{Point3,Vector3};
    ///
    /// let origin = Point3::new(0.0,0.0,0.0);
    /// let direction = Vector3::new(1.0,0.0,0.0);
    /// let ray = Ray::new(origin, direction);
    ///
    /// let point1 = Point3::new(99.9,-1.0,-1.0);
    /// let point2 = Point3::new(100.1,1.0,1.0);
    /// let aabb = AABB::with_bounds(point1, point2);
    ///
    /// assert!(ray.intersects_aabb_branchless(&aabb));
    /// ```
    ///
    /// [`Ray`]: struct.Ray.html
    /// [`AABB`]: struct.AABB.html
    ///
    pub fn intersects_aabb_branchless(&self, aabb: &AABB) -> bool {
        let tx1 = (aabb.min.x - self.origin.x) * self.inv_direction.x;
        let tx2 = (aabb.max.x - self.origin.x) * self.inv_direction.x;

        let mut tmin = tx1.min(tx2);
        let mut tmax = tx1.max(tx2);

        let ty1 = (aabb.min.y - self.origin.y) * self.inv_direction.y;
        let ty2 = (aabb.max.y - self.origin.y) * self.inv_direction.y;

        tmin = tmin.max(ty1.min(ty2));
        tmax = tmax.min(ty1.max(ty2));

        let tz1 = (aabb.min.z - self.origin.z) * self.inv_direction.z;
        let tz2 = (aabb.max.z - self.origin.z) * self.inv_direction.z;

        tmin = tmin.max(tz1.min(tz2));
        tmax = tmax.min(tz1.max(tz2));

        tmax >= tmin && tmax >= 0.0
    }

    /// Implementation of the [Möller-Trumbore triangle/ray intersection algorithm]
    /// (https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm).
    /// Returns the distance to the intersection, as well as
    /// the u and v coordinates of the intersection.
    /// The distance is set to +INFINITY if the ray does not intersect the triangle, or hits
    /// it from behind.
    #[allow(clippy::many_single_char_names)]
    pub fn intersects_triangle(&self, a: &Point3, b: &Point3, c: &Point3) -> Intersection {
        let a_to_b = *b - *a;
        let a_to_c = *c - *a;

        // Begin calculating determinant - also used to calculate u parameter
        // u_vec lies in view plane
        // length of a_to_c in view_plane = |u_vec| = |a_to_c|*sin(a_to_c, dir)
        let u_vec = self.direction.cross(a_to_c);

        // If determinant is near zero, ray lies in plane of triangle
        // The determinant corresponds to the parallelepiped volume:
        // det = 0 => [dir, a_to_b, a_to_c] not linearly independant
        let det = a_to_b.dot(u_vec);

        // Only testing positive bound, thus enabling backface culling
        // If backface culling is not desired write:
        // det < EPSILON && det > -EPSILON
        if det < EPSILON {
            return Intersection::new(Real::INFINITY, 0.0, 0.0, Vector3::ZERO, false);
        }

        let inv_det = 1.0 / det;

        // Vector from point a to ray origin
        let a_to_origin = self.origin - *a;

        // Calculate u parameter
        let u = a_to_origin.dot(u_vec) * inv_det;

        // Test bounds: u < 0 || u > 1 => outside of triangle
        if !(0.0..=1.0).contains(&u) {
            return Intersection::new(Real::INFINITY, u, 0.0, Vector3::ZERO, false);
        }

        // Prepare to test v parameter
        let v_vec = a_to_origin.cross(a_to_b);

        // Calculate v parameter and test bound
        let v = self.direction.dot(v_vec) * inv_det;
        // The intersection lies outside of the triangle
        if v < 0.0 || u + v > 1.0 {
            return Intersection::new(Real::INFINITY, u, v, Vector3::ZERO, false);
        }

        let dist = a_to_c.dot(v_vec) * inv_det;

        if dist > EPSILON {
            let mut normal = Vector3::ZERO;
            normal.x = (a_to_b.y * a_to_c.z) - (a_to_b.z * a_to_c.y);
            normal.y = (a_to_b.z * a_to_c.x) - (a_to_b.x * a_to_c.z);
            normal.z = (a_to_b.x * a_to_c.y) - (a_to_b.y * a_to_c.x);
            Intersection::new(dist, u, v, normal, false)
        } else {
            Intersection::new(Real::INFINITY, u, v, Vector3::ZERO, false)
        }
    }

    /// Returns the t_min of the aabb intersection
    pub fn intersects_aabb_dist(&self, aabb: &AABB) -> Option<Real> {
        let x_min = (aabb[self.sign_x].x - self.origin.x) * self.inv_direction.x;
        let x_max = (aabb[1 - self.sign_x].x - self.origin.x) * self.inv_direction.x;
        let mut ray_min = x_min;
        let mut ray_max = x_max;

        let y_min = (aabb[self.sign_y].y - self.origin.y) * self.inv_direction.y;
        let y_max = (aabb[1 - self.sign_y].y - self.origin.y) * self.inv_direction.y;

        if (ray_min > y_max) || (y_min > ray_max) {
            return None;
        }

        if y_min > ray_min {
            ray_min = y_min;
        }
        // Using the following solution significantly decreases the performance
        // ray_min = ray_min.max(y_min);

        if y_max < ray_max {
            ray_max = y_max;
        }
        // Using the following solution significantly decreases the performance
        // ray_max = ray_max.min(y_max);

        let z_min = (aabb[self.sign_z].z - self.origin.z) * self.inv_direction.z;
        let z_max = (aabb[1 - self.sign_z].z - self.origin.z) * self.inv_direction.z;

        if (ray_min > z_max) || (z_min > ray_max) {
            return None;
        }

        // Only required for bounded intersection intervals.
        // if z_min > ray_min {
        // ray_min = z_min;
        // }

        if z_max < ray_max {
            ray_max = z_max;
        }
        // Using the following solution significantly decreases the performance
        // ray_max = ray_max.min(y_max);

        if ray_max < 0.0 {
            None
        } else {
            let min = Vector3::new(x_min, y_min, z_min).length_squared();
            Some(min)
        }
    }

    /// Returns the position the front of the `Ray` is after traveling dist
    pub fn at(&self, dist: Real) -> Vector3 {
        self.origin + (self.direction * dist)
    }

    /// Given an outward normal returns whether it hit a back_face and adjusts the normal
    pub fn face_normal(&self, out_norm: Vector3) -> (Vector3, bool) {
        let back_face = self.direction.dot(out_norm) >= 0.;
        let norm = if back_face { -out_norm } else { out_norm };
        (norm, back_face)
    }
}

#[cfg(test)]
mod tests {
    use crate::Real;
    use std::cmp;

    use crate::aabb::AABB;
    use crate::bounding_hierarchy::IntersectionAABB;
    use crate::ray::Ray;
    use crate::testbase::{tuple_to_point, tuplevec_small_strategy, TupleVec};
    use crate::EPSILON;
    use proptest::prelude::*;

    /// Generates a random `Ray` which points at at a random `AABB`.
    fn gen_ray_to_aabb(data: (TupleVec, TupleVec, TupleVec)) -> (Ray, AABB) {
        // Generate a random AABB
        let aabb = AABB::empty()
            .grow(&tuple_to_point(&data.0))
            .grow(&tuple_to_point(&data.1));

        // Get its center
        let center = aabb.center();

        // Generate random ray pointing at the center
        let pos = tuple_to_point(&data.2);
        let ray = Ray::new(pos, center - pos);
        (ray, aabb)
    }

    #[cfg(not(miri))]
    proptest! {
        // Test whether a `Ray` which points at the center of an `AABB` intersects it.
        // Uses the optimized algorithm.
        #[test]
        fn test_ray_points_at_aabb_center(data in (tuplevec_small_strategy(),
                                                   tuplevec_small_strategy(),
                                                   tuplevec_small_strategy())) {
            let (ray, aabb) = gen_ray_to_aabb(data);
            assert!(ray.intersects_aabb(&aabb));
        }

        // Test whether a `Ray` which points at the center of an `AABB` intersects it.
        // Uses the naive algorithm.
        #[test]
        fn test_ray_points_at_aabb_center_naive(data in (tuplevec_small_strategy(),
                                                         tuplevec_small_strategy(),
                                                         tuplevec_small_strategy())) {
            let (ray, aabb) = gen_ray_to_aabb(data);
            assert!(ray.intersects_aabb_naive(&aabb));
        }

        // Test whether a `Ray` which points at the center of an `AABB` intersects it.
        // Uses the branchless algorithm.
        #[test]
        fn test_ray_points_at_aabb_center_branchless(data in (tuplevec_small_strategy(),
                                                              tuplevec_small_strategy(),
                                                              tuplevec_small_strategy())) {
            let (ray, aabb) = gen_ray_to_aabb(data);
            assert!(ray.intersects_aabb_branchless(&aabb));
        }

        // Test whether a `Ray` which points away from the center of an `AABB`
        // does not intersect it, unless its origin is inside the `AABB`.
        // Uses the optimized algorithm.
        #[test]
        fn test_ray_points_from_aabb_center(data in (tuplevec_small_strategy(),
                                                     tuplevec_small_strategy(),
                                                     tuplevec_small_strategy())) {
            let (mut ray, aabb) = gen_ray_to_aabb(data);

            // Invert the direction of the ray
            ray.direction = -ray.direction;
            ray.inv_direction = -ray.inv_direction;
            assert!(!ray.intersects_aabb(&aabb) || aabb.contains(&ray.origin));
        }

        // Test whether a `Ray` which points away from the center of an `AABB`
        // does not intersect it, unless its origin is inside the `AABB`.
        // Uses the naive algorithm.
        #[test]
        fn test_ray_points_from_aabb_center_naive(data in (tuplevec_small_strategy(),
                                                           tuplevec_small_strategy(),
                                                           tuplevec_small_strategy())) {
            let (mut ray, aabb) = gen_ray_to_aabb(data);

            // Invert the ray direction
            ray.direction = -ray.direction;
            ray.inv_direction = -ray.inv_direction;
            assert!(!ray.intersects_aabb_naive(&aabb) || aabb.contains(&ray.origin));
        }

        // Test whether a `Ray` which points away from the center of an `AABB`
        // does not intersect it, unless its origin is inside the `AABB`.
        // Uses the branchless algorithm.
        #[test]
        fn test_ray_points_from_aabb_center_branchless(data in (tuplevec_small_strategy(),
                                                                tuplevec_small_strategy(),
                                                                tuplevec_small_strategy())) {
            let (mut ray, aabb) = gen_ray_to_aabb(data);
            // Invert the ray direction
            ray.direction = -ray.direction;
            ray.inv_direction = -ray.inv_direction;
            assert!(!ray.intersects_aabb_branchless(&aabb) || aabb.contains(&ray.origin));
        }

        // Test whether a `Ray` which points at the center of a triangle
        // intersects it, unless it sees the back face, which is culled.
        #[test]
        fn test_ray_hits_triangle(a in tuplevec_small_strategy(),
                                  b in tuplevec_small_strategy(),
                                  c in tuplevec_small_strategy(),
                                  origin in tuplevec_small_strategy(),
                                  u: u16,
                                  v: u16) {
            // Define a triangle, u/v vectors and its normal
            let triangle = (tuple_to_point(&a), tuple_to_point(&b), tuple_to_point(&c));
            let u_vec = triangle.1 - triangle.0;
            let v_vec = triangle.2 - triangle.0;
            let normal = u_vec.cross(v_vec);

            // Get some u and v coordinates such that u+v <= 1
            let u = u % 101;
            let v = cmp::min(100 - u, v % 101);
            let u = u as Real / 100.0;
            let v = v as Real / 100.0;

            // Define some point on the triangle
            let point_on_triangle = triangle.0 + u * u_vec + v * v_vec;

            // Define a ray which points at the triangle
            let origin = tuple_to_point(&origin);
            let ray = Ray::new(origin, point_on_triangle - origin);
            let on_back_side = normal.dot(ray.origin - triangle.0) <= 0.0;

            // Perform the intersection test
            let intersects = ray.intersects_triangle(&triangle.0, &triangle.1, &triangle.2);
            let uv_sum = intersects.u + intersects.v;

            // Either the intersection is in the back side (including the triangle-plane)
            if on_back_side {
                // Intersection must be INFINITY, u and v are undefined
                assert!(intersects.distance == Real::INFINITY);
            } else {
                // Or it is on the front side
                // Either the intersection is inside the triangle, which it should be
                // for all u, v such that u+v <= 1.0
                let intersection_inside = (0.0..=1.0).contains(&uv_sum) && intersects.distance < Real::INFINITY;

                // Or the input data was close to the border
                let close_to_border =
                    u.abs() < EPSILON || (u - 1.0).abs() < EPSILON || v.abs() < EPSILON ||
                    (v - 1.0).abs() < EPSILON || (u + v - 1.0).abs() < EPSILON;

                if !(intersection_inside || close_to_border) {
                    println!("uvsum {}", uv_sum);
                    println!("intersects.0 {}", intersects.distance);
                    println!("intersects.1 (u) {}", intersects.u);
                    println!("intersects.2 (v) {}", intersects.v);
                    println!("u {}", u);
                    println!("v {}", v);
                }

                assert!(intersection_inside || close_to_border);
            }
        }
    }
}

#[cfg(all(feature = "bench", test))]
mod bench {
    use crate::bounding_hierarchy::IntersectionAABB;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    use crate::aabb::AABB;
    use crate::ray::Ray;

    use crate::testbase::{tuple_to_point, tuple_to_vector, TupleVec};

    /// Generates some random deterministic `Ray`/`AABB` pairs.
    fn gen_random_ray_aabb(rng: &mut StdRng) -> (Ray, AABB) {
        let a = tuple_to_point(&rng.gen::<TupleVec>());
        let b = tuple_to_point(&rng.gen::<TupleVec>());
        let c = tuple_to_point(&rng.gen::<TupleVec>());
        let d = tuple_to_vector(&rng.gen::<TupleVec>());

        let aabb = AABB::empty().grow(&a).grow(&b);
        let ray = Ray::new(c, d);
        (ray, aabb)
    }

    /// Benchmark for the optimized intersection algorithm.
    #[bench]
    fn bench_intersects_aabb(b: &mut ::test::Bencher) {
        let seed = [0; 32];
        let mut rng = StdRng::from_seed(seed);

        b.iter(|| {
            let one_thousand = ::test::black_box(1000);
            for _ in 0..one_thousand {
                let (ray, aabb) = gen_random_ray_aabb(&mut rng);
                ray.intersects_aabb(&aabb);
            }
        });
    }

    /// Benchmark for the naive intersection algorithm.
    #[bench]
    fn bench_intersects_aabb_naive(b: &mut ::test::Bencher) {
        let seed = [0; 32];
        let mut rng = StdRng::from_seed(seed);

        b.iter(|| {
            let one_thousand = ::test::black_box(1000);
            for _ in 0..one_thousand {
                let (ray, aabb) = gen_random_ray_aabb(&mut rng);
                ray.intersects_aabb_naive(&aabb);
            }
        });
    }

    /// Benchmark for the branchless intersection algorithm.
    #[bench]
    fn bench_intersects_aabb_branchless(b: &mut ::test::Bencher) {
        let seed = [0; 32];
        let mut rng = StdRng::from_seed(seed);

        b.iter(|| {
            let one_thousand = ::test::black_box(1000);
            for _ in 0..one_thousand {
                let (ray, aabb) = gen_random_ray_aabb(&mut rng);
                ray.intersects_aabb_branchless(&aabb);
            }
        });
    }
}
