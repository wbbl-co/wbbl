use glam::{Mat2, Vec2};
use std::{cmp::Ordering, collections::VecDeque};
use wasm_bindgen::prelude::*;

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
            if c > 0.0 || (c == 0.0 && p.length_squared() <= current.length_squared()) {
                break;
            }
            stack.pop_front();
        }
        stack.push_front(p.clone());
    }
    return stack.into_iter().collect();
}

pub fn get_ray_ray_intersection(
    start_1: &Vec2,
    end_1: &Vec2,
    start_2: &Vec2,
    end_2: &Vec2,
) -> Option<Vec2> {
    let current_line_mat = Mat2::from_cols(start_1.clone(), end_1.clone())
        .transpose()
        .determinant();
    let next_line_mat = Mat2::from_cols(start_2.clone(), end_2.clone())
        .transpose()
        .determinant();
    let current_line_xs = Mat2::from_cols(Vec2::new(start_1.x, end_1.x), Vec2::splat(1.0))
        .transpose()
        .determinant();
    let current_line_ys = Mat2::from_cols(Vec2::new(start_1.y, end_1.y), Vec2::splat(1.0))
        .transpose()
        .determinant();
    let next_line_xs = Mat2::from_cols(Vec2::new(start_2.x, end_2.x), Vec2::splat(1.0))
        .transpose()
        .determinant();
    let next_line_ys = Mat2::from_cols(Vec2::new(start_2.y, end_2.y), Vec2::splat(1.0))
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

    if denominator != 0.0 {
        return Some(Vec2::new(
            x_numerator / denominator,
            y_numerator / denominator,
        ));
    }
    None
}

pub fn get_line_line_intersection(
    start_1: &Vec2,
    end_1: &Vec2,
    start_2: &Vec2,
    end_2: &Vec2,
) -> Option<Vec2> {
    if let Some(intersection) = get_ray_ray_intersection(start_1, end_1, start_2, end_2) {
        let start_1_intersection = intersection.clone() - start_1.clone();
        let start_2_intersection = intersection.clone() - start_2.clone();
        let line_1 = end_1.clone() - start_1.clone();
        let line_2 = end_2.clone() - start_2.clone();
        let t_1 = start_1_intersection.length() / line_1.length();
        let t_2 = start_2_intersection.length() / line_2.length();

        if t_1 <= 1.0
            && t_2 <= 1.0
            && line_1.dot(start_1_intersection) >= 0.0
            && line_2.dot(start_2_intersection) >= 0.0
        {
            Some(intersection)
        } else {
            None
        }
    } else {
        None
    }
}

#[wasm_bindgen]
pub fn is_axis_aligned_rect_intersecting_convex_hull(
    hull: &[f32],
    top_left: &[f32],
    bottom_right: &[f32],
) -> bool {
    let top_left = Vec2::from_slice(top_left);
    let bottom_right = Vec2::from_slice(bottom_right);
    let hull: Vec<Vec2> = hull
        .iter()
        .zip(hull.iter().skip(1))
        .enumerate()
        .filter(|(i, _)| i % 2 == 0)
        .map(|(_, (x, y))| Vec2::new(*x, *y))
        .collect();
    // Case where the hull is completely enveloped by the bounding box
    if hull.iter().all(|p| top_left.x <= p.x && top_left.y <= p.y)
        && hull
            .iter()
            .all(|p| bottom_right.x >= p.x && bottom_right.y >= p.y)
    {
        return true;
    }

    let top_right = Vec2::new(bottom_right.x, top_left.y);
    let bottom_left = Vec2::new(top_left.x, bottom_right.y);

    for i in 1..hull.len() {
        let prev = hull[i - 1];
        let current = hull[i];
        if get_line_line_intersection(&bottom_right, &top_right, &prev, &current).is_some() {
            return true;
        }
        if get_line_line_intersection(&bottom_left, &top_left, &prev, &current).is_some() {
            return true;
        }
        if get_line_line_intersection(&top_left, &top_right, &prev, &current).is_some() {
            return true;
        }
        if get_line_line_intersection(&bottom_left, &bottom_right, &prev, &current).is_some() {
            return true;
        }
    }
    // Code to test whether inside shape
    // hull.push(top_left.clone());
    // hull.push(bottom_right.clone());
    // hull.push(top_right.clone());
    // hull.push(bottom_left.clone());
    // let hull = get_convex_hull(&mut hull);
    // if hull.contains(&top_left)
    //     || hull.contains(&top_right)
    //     || hull.contains(&top_left)
    //     || hull.contains(&bottom_left)
    // {
    //     return false;
    // }

    false
}
