use std::{cmp::Ordering, collections::VecDeque};

use glam::{Mat2, Vec2};
use wgpu::naga::front;

use crate::log;

fn ccw(a: &Vec2, b: &Vec2, c: &Vec2) -> f32 {
    let delta_ba = b.extend(0.0) - a.extend(0.0);
    let delta_ca = c.extend(0.0) - a.extend(0.0);
    delta_ba.cross(delta_ca).z
}

pub fn get_convex_hull(points: &mut Vec<Vec2>) -> Vec<Vec2> {
    if points.len() == 0 {
        return vec![];
    }
    let mut lowest_point = Vec2::MAX;
    for p in points.iter() {
        if p.y < lowest_point.y {
            lowest_point = p.clone();
        } else if p.y == lowest_point.y && p.x < lowest_point.x {
            lowest_point = p.clone();
        }
    }
    let x_axis = Vec2::new(1.0, 0.0);
    points.sort_unstable_by(|a, b| {
        let delta_ap = *a - lowest_point;
        let delta_bp = *b - lowest_point;
        let manhattan_ap = f32::abs(delta_ap.x) + f32::abs(delta_ap.y);
        let manhattan_bp = f32::abs(delta_bp.x) + f32::abs(delta_bp.y);
        if manhattan_ap == 0.0 {
            return Ordering::Less;
        }
        if manhattan_bp == 0.0 {
            return Ordering::Greater;
        }
        let dot_ap = delta_ap.normalize().dot(x_axis);
        let dot_bp = delta_bp.normalize().dot(x_axis);
        if dot_ap < dot_bp {
            Ordering::Greater
        } else if dot_ap == dot_bp {
            // Furtherest points go first
            if manhattan_ap > manhattan_bp {
                Ordering::Less
            } else if manhattan_ap == manhattan_bp {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        } else {
            Ordering::Less
        }
    });

    let mut stack = VecDeque::new();
    for p in points.iter() {
        while stack.len() > 1 {
            let next = stack.get(1).unwrap();
            let current = stack.front().unwrap();
            let c = ccw(next, current, p);
            if c > 0.0 || (c == 0.0 && p.length_squared() < current.length_squared()) {
                break;
            }
            stack.pop_front();
        }
        stack.push_front(p.clone());
    }
    return stack.into_iter().collect();
}

pub fn get_line_line_intersection(
    prev: &Vec2,
    current: &Vec2,
    next: &Vec2,
    next_next: &Vec2,
) -> Vec2 {
    let current_line_mat = Mat2::from_cols(prev.clone(), current.clone())
        .transpose()
        .determinant();
    let next_line_mat = Mat2::from_cols(next.clone(), next_next.clone())
        .transpose()
        .determinant();
    let current_line_xs = Mat2::from_cols(Vec2::new(prev.x, current.x), Vec2::splat(1.0))
        .transpose()
        .determinant();
    let current_line_ys = Mat2::from_cols(Vec2::new(prev.y, current.y), Vec2::splat(1.0))
        .transpose()
        .determinant();
    let next_line_xs = Mat2::from_cols(Vec2::new(next.x, next_next.x), Vec2::splat(1.0))
        .transpose()
        .determinant();
    let next_line_ys = Mat2::from_cols(Vec2::new(next.y, next_next.y), Vec2::splat(1.0))
        .transpose()
        .determinant();
    let x_numerator = Mat2::from_cols(
        Vec2::new(current_line_mat, next_line_mat),
        Vec2::new(current_line_xs, next_line_xs),
    )
    .determinant();
    let y_numerator = Mat2::from_cols(
        Vec2::new(current_line_mat, next_line_mat),
        Vec2::new(current_line_ys, next_line_ys),
    )
    .determinant();
    let denominator = Mat2::from_cols(
        Vec2::new(current_line_xs, next_line_xs),
        Vec2::new(current_line_ys, next_line_ys),
    )
    .determinant();

    Vec2::new(x_numerator / denominator, y_numerator / denominator)
}
