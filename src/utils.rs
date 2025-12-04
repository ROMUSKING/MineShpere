use bevy::prelude::*;
use bevy::render::{mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology};
use std::collections::HashMap;
use std::collections::HashSet;

// --- GEOMETRY UTILS ---

pub fn generate_goldberg_polyhedron(radius: f32, subdivisions: usize) -> (Vec<Vec<Vec3>>, Vec<Vec<usize>>) {
    let t = (1.0 + 5.0f32.sqrt()) / 2.0;
    let mut verts = vec![
        Vec3::new(-1.0, t, 0.0), Vec3::new(1.0, t, 0.0), Vec3::new(-1.0, -t, 0.0), Vec3::new(1.0, -t, 0.0),
        Vec3::new(0.0, -1.0, t), Vec3::new(0.0, 1.0, t), Vec3::new(0.0, -1.0, -t), Vec3::new(0.0, 1.0, -t),
        Vec3::new(t, 0.0, -1.0), Vec3::new(t, 0.0, 1.0), Vec3::new(-t, 0.0, -1.0), Vec3::new(-t, 0.0, 1.0),
    ];
    for v in &mut verts { *v = v.normalize(); }

    let mut faces = vec![
        vec![0, 11, 5], vec![0, 5, 1], vec![0, 1, 7], vec![0, 7, 10], vec![0, 10, 11],
        vec![1, 5, 9], vec![5, 11, 4], vec![11, 10, 2], vec![10, 7, 6], vec![7, 1, 8],
        vec![3, 9, 4], vec![3, 4, 2], vec![3, 2, 6], vec![3, 6, 8], vec![3, 8, 9],
        vec![4, 9, 5], vec![2, 4, 11], vec![6, 2, 10], vec![8, 6, 7], vec![9, 8, 1],
    ];

    for _ in 0..subdivisions {
        let mut next_faces = Vec::new();
        let mut mid_cache = HashMap::new();
        for f in faces {
            let (v1, v2, v3) = (f[0], f[1], f[2]);
            let a = get_midpoint(v1, v2, &mut verts, &mut mid_cache);
            let b = get_midpoint(v2, v3, &mut verts, &mut mid_cache);
            let c = get_midpoint(v3, v1, &mut verts, &mut mid_cache);
            next_faces.extend_from_slice(&[vec![v1, a, c], vec![v2, b, a], vec![v3, c, b], vec![a, b, c]]);
        }
        faces = next_faces;
    }
    
    let mut poly_map: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut centers = Vec::new();
    for (i, f) in faces.iter().enumerate() {
        centers.push(((verts[f[0]] + verts[f[1]] + verts[f[2]]) / 3.0).normalize() * radius);
        for &v in f { poly_map.entry(v).or_default().push(i); }
    }

    let mut polygons = Vec::new();
    let mut adjacency = Vec::new();

    for i in 0..verts.len() {
        if let Some(indices) = poly_map.get(&i) {
            let center = verts[i];
            let up = center.normalize();
            let mut sorted = indices.clone();
            sorted.sort_by(|&a, &b| {
                let pa = centers[a] - center * radius;
                let pb = centers[b] - center * radius;
                let tan = if up.y.abs() > 0.9 { Vec3::X } else { Vec3::Y }.cross(up).normalize();
                let bitan = up.cross(tan);
                pa.dot(tan).atan2(pa.dot(bitan)).partial_cmp(&pb.dot(tan).atan2(pb.dot(bitan))).unwrap()
            });
            polygons.push(sorted.iter().map(|&idx| centers[idx]).collect());
            
            let mut neighbors = HashSet::new();
            for &fi in &sorted {
                for &v in &faces[fi] {
                    if v != i { neighbors.insert(v); }
                }
            }
            adjacency.push(neighbors.into_iter().collect());
        }
    }
    (polygons, adjacency)
}

fn get_midpoint(p1: usize, p2: usize, verts: &mut Vec<Vec3>, cache: &mut HashMap<(usize, usize), usize>) -> usize {
    let key = if p1 < p2 { (p1, p2) } else { (p2, p1) };
    if let Some(&idx) = cache.get(&key) { return idx; }
    verts.push(((verts[p1] + verts[p2]) * 0.5).normalize());
    cache.insert(key, verts.len() - 1);
    verts.len() - 1
}

pub fn create_polygon_mesh(verts: &Vec<Vec3>) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let center = verts.iter().sum::<Vec3>() / verts.len() as f32;
    let mut pos: Vec<[f32; 3]> = vec![center.into()];
    let mut norm: Vec<[f32; 3]> = vec![center.normalize().into()];
    let mut idxs = Vec::new();

    for (i, v) in verts.iter().enumerate() {
        let v_gap = center + (*v - center) * 0.92;
        pos.push(v_gap.into());
        norm.push(v_gap.normalize().into());
        let next = (i + 1) % verts.len();
        idxs.extend_from_slice(&[0, (next + 1) as u32, (i + 1) as u32]);
    }
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, norm);
    mesh.insert_indices(Indices::U32(idxs));
    mesh
}
