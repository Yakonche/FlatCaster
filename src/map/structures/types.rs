// src/map/structures/types.rs

/// A wall segment: (x1, y1, x2, y2, color_type)
pub type Segment = (f32, f32, f32, f32, u32);
/// Bounding box: (x, y, w, h)
pub type BBox = (f32, f32, f32, f32);

pub fn check_overlap(bboxes: &[BBox], new_box: BBox) -> bool {
    let (nx, ny, nw, nh) = new_box;
    for &(bx, by, bw, bh) in bboxes {
        if !(nx + nw < bx || nx > bx + bw || ny + nh < by || ny > by + bh) {
            return false;
        }
    }
    true
}

/// Disjoint Set (Union-Find) for Kruskal's Algorithm
pub struct DisjointSet {
    parent: Vec<usize>,
}

impl DisjointSet {
    pub fn new(size: usize) -> Self {
        Self { parent: (0..size).collect() }
    }

    pub fn find(&mut self, i: usize) -> usize {
        if self.parent[i] == i { i } else {
            let p = self.parent[i];
            self.parent[i] = self.find(p);
            self.parent[i]
        }
    }

    pub fn union(&mut self, i: usize, j: usize) {
        let root_i = self.find(i);
        let root_j = self.find(j);
        if root_i != root_j { self.parent[root_i] = root_j; }
    }
}
