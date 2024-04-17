use glam::Vec2;
use mint::Point2;
use rstar::{RTreeObject, AABB};

#[derive(Debug, PartialEq)]
pub enum GraphSpatialLookupType {
    Node(u128, Vec2, Vec2),
    Edge(u128, Vec2, Vec2),
    Group(u128, Vec<Vec2>),
}

impl RTreeObject for GraphSpatialLookupType {
    type Envelope = AABB<Point2<f32>>;
    fn envelope(&self) -> Self::Envelope {
        match self {
            GraphSpatialLookupType::Node(_, top_left, bottom_right) => {
                AABB::from_corners(top_left.clone().into(), bottom_right.clone().into())
            }
            GraphSpatialLookupType::Edge(_, a, b) => {
                AABB::from_points(&[a.clone().into(), b.clone().into()])
            }
            GraphSpatialLookupType::Group(_, points) => {
                let points: Vec<Point2<f32>> = points
                    .iter()
                    .map(|p: &Vec2| -> Point2<f32> { p.clone().into() })
                    .collect();
                AABB::from_points(&points)
            }
        }
    }
}
