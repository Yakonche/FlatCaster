// src/map/structures/solid_rect.rs
use super::types::{Segment, BBox, check_overlap};

pub fn generate_solid_rect(
    segments: &mut Vec<Segment>,
    bboxes: &mut Vec<BBox>,
    x: f32, y: f32, w: f32, h: f32, color: u32,
) -> bool {
    if !check_overlap(bboxes, (x - 1.0, y - 1.0, w + 2.0, h + 2.0)) { return false; }
    bboxes.push((x - 1.0, y - 1.0, w + 2.0, h + 2.0));
    segments.push((x, y, x + w, y, color));
    segments.push((x + w, y, x + w, y + h, color));
    segments.push((x + w, y + h, x, y + h, color));
    segments.push((x, y + h, x, y, color));
    true
}
