// src/map/structures/hex_maze.rs
use rand::Rng;
use std::collections::HashSet;
use super::types::{Segment, BBox, check_overlap};

pub fn generate_hex_maze(
    segments: &mut Vec<Segment>,
    bboxes: &mut Vec<BBox>,
    cx: f32, cy: f32,
    radius_cells: usize,
    hex_size: f32,
    color: u32,
    rng: &mut impl Rng,
) -> bool {
    let total_r = radius_cells as f32 * hex_size * 2.0;
    if !check_overlap(bboxes, (cx - total_r, cy - total_r, total_r * 2.0, total_r * 2.0)) {
        return false;
    }
    bboxes.push((cx - total_r, cy - total_r, total_r * 2.0, total_r * 2.0));

    let mut visited: HashSet<(i32, i32)> = HashSet::new();
    let mut stack = Vec::new();
    let mut removed_walls: HashSet<((i32, i32), (i32, i32))> = HashSet::new();

    stack.push((0, 0));
    visited.insert((0, 0));

    let directions = [
        (1, 0), (1, -1), (0, -1),
        (-1, 0), (-1, 1), (0, 1)
    ];

    while let Some(current) = stack.last().copied() {
        let (q, r) = current;
        let mut neighbors = Vec::new();

        for (dq, dr) in directions.iter() {
            let nq: i32 = q + dq;
            let nr: i32 = r + dr;
            let dist = (nq.abs() + (nq + nr).abs() + nr.abs()) / 2;
            if dist < radius_cells as i32 {
                if !visited.contains(&(nq, nr)) {
                    neighbors.push((nq, nr));
                }
            }
        }

        if neighbors.is_empty() {
            stack.pop();
        } else {
            let next = neighbors[rng.gen_range(0..neighbors.len())];
            let edge = if current < next { (current, next) } else { (next, current) };
            removed_walls.insert(edge);
            visited.insert(next);
            stack.push(next);
        }
    }

    for &(q, r) in &visited {
        let center_x = cx + hex_size * (3.0f32.sqrt() * q as f32 + 3.0f32.sqrt()/2.0 * r as f32);
        let center_y = cy + hex_size * (3.0/2.0 * r as f32);

        for (i, (dq, dr)) in directions.iter().enumerate() {
            let nq: i32 = q + dq;
            let nr: i32 = r + dr;
            let neighbor = (nq, nr);

            let edge = if (q, r) < neighbor { ((q, r), neighbor) } else { (neighbor, (q, r)) };
            let draw_wall = !visited.contains(&neighbor) || !removed_walls.contains(&edge);

            if draw_wall {
                let angle_deg = 60.0 * i as f32 + 30.0;
                let a1 = (angle_deg - 30.0).to_radians();
                let a2 = (angle_deg + 30.0).to_radians();

                let x1 = center_x + hex_size * a1.cos();
                let y1 = center_y + hex_size * a1.sin();
                let x2 = center_x + hex_size * a2.cos();
                let y2 = center_y + hex_size * a2.sin();

                segments.push((x1, y1, x2, y2, color));
            }
        }
    }

    true
}
