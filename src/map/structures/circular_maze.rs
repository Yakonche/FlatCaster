// src/map/structures/circular_maze.rs
use rand::Rng;
use std::f32::consts::PI;
use super::types::{Segment, BBox, check_overlap};

pub fn generate_circular_maze(
    segments: &mut Vec<Segment>,
    bboxes: &mut Vec<BBox>,
    cx: f32, cy: f32,
    num_rings: usize,
    sectors: usize,
    radius_step: f32,
    color: u32,
    rng: &mut impl Rng,
) -> bool {
    let total_radius = num_rings as f32 * radius_step;
    if !check_overlap(bboxes, (cx - total_radius - 1.0, cy - total_radius - 1.0, total_radius * 2.0 + 2.0, total_radius * 2.0 + 2.0)) {
        return false;
    }
    bboxes.push((cx - total_radius - 1.0, cy - total_radius - 1.0, total_radius * 2.0 + 2.0, total_radius * 2.0 + 2.0));

    let mut radial_walls = vec![vec![true; sectors]; num_rings];
    let mut arc_walls = vec![vec![true; sectors]; num_rings + 1];
    let mut visited = vec![vec![false; sectors]; num_rings];

    let start_r = 0;
    let start_s = 0;
    visited[start_r][start_s] = true;
    let mut stack = vec![(start_r, start_s)];

    while let Some(&(cr, cs)) = stack.last() {
        let mut neighbors = Vec::new();
        if cr + 1 < num_rings && !visited[cr + 1][cs] { neighbors.push((cr + 1, cs, "out")); }
        if cr > 0 && !visited[cr - 1][cs] { neighbors.push((cr - 1, cs, "in")); }
        let next_s = (cs + 1) % sectors;
        if !visited[cr][next_s] { neighbors.push((cr, next_s, "ccw")); }
        let prev_s = if cs == 0 { sectors - 1 } else { cs - 1 };
        if !visited[cr][prev_s] { neighbors.push((cr, prev_s, "cw")); }

        if neighbors.is_empty() {
            stack.pop();
        } else {
            let (nr, ns, dir) = neighbors[rng.gen_range(0..neighbors.len())];
            match dir {
                "out" => arc_walls[nr][cs] = false,
                "in" => arc_walls[cr][cs] = false,
                "ccw" => radial_walls[cr][cs] = false,
                "cw" => radial_walls[cr][prev_s] = false,
                _ => {}
            }
            visited[nr][ns] = true;
            stack.push((nr, ns));
        }
    }

    arc_walls[0][0] = false;
    arc_walls[num_rings][rng.gen_range(0..sectors)] = false;

    for r in 0..=num_rings {
        for s in 0..sectors {
            if arc_walls[r][s] {
                if r == 0 { continue; }
                let radius = r as f32 * radius_step;
                let theta_start = (s as f32 / sectors as f32) * 2.0 * PI;
                let theta_end = ((s + 1) as f32 / sectors as f32) * 2.0 * PI;

                let x1 = cx + theta_start.cos() * radius;
                let y1 = cy + theta_start.sin() * radius;
                let x2 = cx + theta_end.cos() * radius;
                let y2 = cy + theta_end.sin() * radius;
                segments.push((x1, y1, x2, y2, color));
            }
        }
    }

    for r in 0..num_rings {
        for s in 0..sectors {
            if radial_walls[r][s] {
                let inner_rad = r as f32 * radius_step;
                let outer_rad = (r + 1) as f32 * radius_step;
                let theta = ((s + 1) as f32 / sectors as f32) * 2.0 * PI;
                let x1 = cx + theta.cos() * inner_rad;
                let y1 = cy + theta.sin() * inner_rad;
                let x2 = cx + theta.cos() * outer_rad;
                let y2 = cy + theta.sin() * outer_rad;
                segments.push((x1, y1, x2, y2, color));
            }
        }
    }
    true
}
