// src/map/structures/rect_maze.rs
use rand::Rng;
use super::types::{Segment, BBox, check_overlap, DisjointSet};

#[derive(Clone, Copy)]
pub enum MazeAlgo {
    RecursiveBacktracker,
    Prim,
    Kruskal,
}

pub fn generate_rectangular_maze(
    segments: &mut Vec<Segment>,
    bboxes: &mut Vec<BBox>,
    start_x: f32, start_y: f32,
    w: usize, h: usize,
    color: u32,
    algo: MazeAlgo,
    rng: &mut impl Rng,
) -> bool {
    if !check_overlap(bboxes, (start_x - 1.0, start_y - 1.0, w as f32 + 2.0, h as f32 + 2.0)) {
        return false;
    }
    bboxes.push((start_x - 1.0, start_y - 1.0, w as f32 + 2.0, h as f32 + 2.0));

    let mut horiz = vec![vec![true; w]; h + 1];
    let mut vert = vec![vec![true; w + 1]; h];

    match algo {
        MazeAlgo::RecursiveBacktracker => {
            let mut visited = vec![vec![false; w]; h];
            let mut stack = vec![(0, 0)];
            visited[0][0] = true;

            while let Some((cx, cy)) = stack.last().copied() {
                let mut neighbors = Vec::new();
                for (dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
                    let nx = cx as i32 + dx;
                    let ny = cy as i32 + dy;
                    if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                        if !visited[ny as usize][nx as usize] {
                            neighbors.push((nx as usize, ny as usize, *dx, *dy));
                        }
                    }
                }

                if neighbors.is_empty() {
                    stack.pop();
                } else {
                    let (nx, ny, dx, dy) = neighbors[rng.gen_range(0..neighbors.len())];
                    if dx == 1 { vert[cy][nx] = false; }
                    else if dx == -1 { vert[cy][cx] = false; }
                    else if dy == 1 { horiz[ny][cx] = false; }
                    else if dy == -1 { horiz[cy][cx] = false; }
                    visited[ny][nx] = true;
                    stack.push((nx, ny));
                }
            }
        },
        MazeAlgo::Prim => {
            let mut visited = vec![vec![false; w]; h];
            let mut frontier = Vec::new();

            let sx = rng.gen_range(0..w);
            let sy = rng.gen_range(0..h);
            visited[sy][sx] = true;

            for (dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
                let nx = sx as i32 + dx;
                let ny = sy as i32 + dy;
                if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                    frontier.push(((nx as usize, ny as usize), (sx, sy)));
                }
            }

            while !frontier.is_empty() {
                let idx = rng.gen_range(0..frontier.len());
                let ((tx, ty), (fx, fy)) = frontier.remove(idx);

                if !visited[ty][tx] {
                    visited[ty][tx] = true;
                    if tx > fx { vert[fy][tx] = false; }
                    else if tx < fx { vert[fy][fx] = false; }
                    else if ty > fy { horiz[ty][tx] = false; }
                    else if ty < fy { horiz[fy][tx] = false; }

                    for (dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
                        let nx = tx as i32 + dx;
                        let ny = ty as i32 + dy;
                        if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                            if !visited[ny as usize][nx as usize] {
                                frontier.push(((nx as usize, ny as usize), (tx, ty)));
                            }
                        }
                    }
                }
            }
        },
        MazeAlgo::Kruskal => {
            let mut set = DisjointSet::new(w * h);
            let mut edges = Vec::new();

            for y in 0..h-1 {
                for x in 0..w {
                    edges.push(((x, y), (x, y+1), "horiz"));
                }
            }
            for y in 0..h {
                for x in 0..w-1 {
                    edges.push(((x, y), (x+1, y), "vert"));
                }
            }

            for i in 0..edges.len() {
                let j = rng.gen_range(0..edges.len());
                edges.swap(i, j);
            }

            for ((x1, y1), (x2, y2), type_) in edges {
                let idx1 = y1 * w + x1;
                let idx2 = y2 * w + x2;
                if set.find(idx1) != set.find(idx2) {
                    set.union(idx1, idx2);
                    if type_ == "horiz" { horiz[y2][x1] = false; }
                    else { vert[y1][x2] = false; }
                }
            }
        }
    }

    horiz[0][0] = false;
    horiz[h][w-1] = false;

    for y in 0..=h {
        for x in 0..w {
            if horiz[y][x] {
                segments.push((
                    start_x + x as f32, start_y + y as f32,
                    start_x + x as f32 + 1.0, start_y + y as f32,
                    color,
                ));
            }
        }
    }
    for y in 0..h {
        for x in 0..=w {
            if vert[y][x] {
                segments.push((
                    start_x + x as f32, start_y + y as f32,
                    start_x + x as f32, start_y + y as f32 + 1.0,
                    color,
                ));
            }
        }
    }

    true
}
