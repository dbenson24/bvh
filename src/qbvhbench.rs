#![cfg(test)]
use parry3d::bounding_volume::Aabb;
use parry3d::math::{Point, Vector};
use parry3d::partitioning::Qbvh;
use parry3d::query::Ray;
use parry3d::query::visitors::RayIntersectionsVisitor;
use crate::aabb::Bounded;
use crate::testbase::{default_bounds, create_n_cubes, create_ray, load_sponza_scene};

#[cfg(feature = "bench")]
#[bench]
fn bench_build_qbvh_120k(b: &mut ::test::Bencher) {
    let bounds = default_bounds();
    let mut triangles: Vec<_> = create_n_cubes(10000, &bounds).into_iter().map(|t| {
        let aabb = t.aabb();
        Aabb::new(aabb.min, aabb.max)
    }).collect();
    b.iter(|| {
        let mut tree = Qbvh::new();
        tree.clear_and_rebuild(triangles.iter().cloned().enumerate(), 0.0);
    });
}

#[cfg(feature = "bench")]
#[bench]
fn bench_intersect_qbvh_120k(b: &mut ::test::Bencher) {
    let bounds = default_bounds();
    let mut triangles: Vec<_> = create_n_cubes(10000, &bounds).into_iter().map(|t| {
        let aabb = t.aabb();
        Aabb::new(aabb.min, aabb.max)
    }).collect();
    let mut tree = Qbvh::new();
    tree.clear_and_rebuild(triangles.iter().cloned().enumerate(), 0.0);
    
    let mut seed = 0;
    b.iter(|| {
        let ray = create_ray(&mut seed, &bounds);
        let ray = Ray::new(ray.origin, ray.direction);
        let mut visit = &mut |_: &usize| true;
        let mut visitor = RayIntersectionsVisitor::new(&ray, 1000000., &mut visit);
        tree.traverse_depth_first(&mut visitor);
    });

}

#[cfg(feature = "bench")]
#[bench]
fn bench_build_qbvh_sponza(b: &mut ::test::Bencher) {
    let (triangles, bounds) = load_sponza_scene();
    let mut triangles: Vec<_> = triangles.into_iter().map(|t| {
        let aabb = t.aabb();
        Aabb::new(aabb.min, aabb.max)
    }).collect();
    
    b.iter(|| {
        let mut tree = Qbvh::new();
        tree.clear_and_rebuild(triangles.iter().cloned().enumerate(), 0.0);
    });
}


#[cfg(feature = "bench")]
#[bench]
fn bench_intersect_qbvh_sponza(b: &mut ::test::Bencher) {
    let (triangles, bounds) = load_sponza_scene();
    let mut triangles: Vec<_> = triangles.into_iter().map(|t| {
        let aabb = t.aabb();
        Aabb::new(aabb.min, aabb.max)
    }).collect();
    
    let mut tree = Qbvh::new();
    tree.clear_and_rebuild(triangles.iter().cloned().enumerate(), 0.0);

    let mut seed = 0;
    b.iter(|| {
        let ray = create_ray(&mut seed, &bounds);
        let ray = Ray::new(ray.origin, ray.direction);
        let mut visit = &mut |_: &usize| true;
        let mut visitor = RayIntersectionsVisitor::new(&ray, 1000000., &mut visit);
        tree.traverse_depth_first(&mut visitor);
    });
}
