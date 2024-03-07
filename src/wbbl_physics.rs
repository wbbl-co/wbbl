use glam::Vec2;
use std::collections::HashMap;

pub trait GetPosition {
    fn get_position(&self) -> Vec2;
}

pub trait SetPosition {
    fn set_position(&mut self, position: Vec2);
}

pub trait Vertlet {
    fn update(&mut self, delta_time: f32, delta_time_squared: f32);
}

pub struct VelocityVertlet {
    pub acceleration: Vec2,
    pub position: Vec2,
    pub velocity: Vec2,
    pub forces: Vec<Force>,
    pub new_acceleration: Vec2,
}

impl VelocityVertlet {
    pub fn gather_forces(
        vertlets: &mut HashMap<u128, VelocityVertlet>,
        forces: &HashMap<u128, Vec<Force>>,
    ) {
        let mut new_accelerations: Vec<(u128, Vec2)> = Vec::new();
        for v in vertlets.iter() {
            let forces = forces.get(v.0).unwrap();
            new_accelerations.push((
                (*v.0),
                forces.iter().fold(Vec2::ZERO, |result, force| {
                    result + force.get_accelleration(vertlets, *v.0)
                }),
            ));
        }
        for (v_id, acceleration) in new_accelerations {
            vertlets
                .entry(v_id)
                .and_modify(|v| v.new_acceleration = acceleration);
        }
    }
}

impl Vertlet for VelocityVertlet {
    fn update(&mut self, delta_time: f32, delta_time_squared: f32) {
        let new_position = self.position
            + self.velocity * Vec2::splat(delta_time)
            + self.acceleration * Vec2::splat(delta_time_squared * 0.5);
        let new_velocity = self.velocity
            + (self.acceleration + self.new_acceleration) * Vec2::splat(delta_time * 0.5);
        self.position = new_position;
        self.velocity = new_velocity;
        self.acceleration = self.new_acceleration;
    }
}

impl GetPosition for VelocityVertlet {
    fn get_position(&self) -> Vec2 {
        self.position
    }
}

impl SetPosition for VelocityVertlet {
    fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }
}

pub struct PositionVertlet {
    pub acceleration: Vec2,
    pub previous_position: Vec2,
    pub position: Vec2,
    pub new_acceleration: Vec2,
}

impl Vertlet for PositionVertlet {
    fn update(&mut self, _delta_time: f32, delta_time_squared: f32) {
        let position_copy: Vec2 = self.position.clone();
        self.position = Vec2::splat(2.0) * self.position - self.previous_position
            + delta_time_squared * self.acceleration;
        self.previous_position = position_copy
    }
}

impl GetPosition for PositionVertlet {
    fn get_position(&self) -> Vec2 {
        self.position
    }
}

impl SetPosition for PositionVertlet {
    fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }
}

pub enum Force {
    Constant(Vec2),
    Damper(f32),
    Spring {
        coefficient: f32,
        target_id: u128,
        target_distance: f32,
        target_offset: Vec2,
    },
}

impl Force {
    pub fn get_accelleration(&self, vertlets: &HashMap<u128, VelocityVertlet>, v: u128) -> Vec2 {
        match self {
            Force::Constant(acceleration) => *acceleration,
            Force::Damper(coefficient) => {
                Vec2::splat(-coefficient) * vertlets.get(&v).unwrap().velocity
            }
            Force::Spring {
                coefficient,
                target_id,
                target_distance,
                target_offset,
            } => {
                let v = vertlets.get(&v).unwrap();
                let t = vertlets.get(&target_id).unwrap();
                let delta = t.position + (*target_offset) - v.position;
                Vec2::splat(-coefficient * target_distance - delta.length())
                    * (delta.normalize_or_zero())
            }
        }
    }
}

pub enum Constraint {
    LockPosition {
        position: Vec2,
        v: u128,
    },
    UnidirectionalDistance {
        v_to_change: u128,
        v_to_get_position_from: u128,
        distance: f32,
    },
    BidirectionalDistance {
        v1: u128,
        v2: u128,
        distance: f32,
    },
    InteractionDragging {
        position: Vec2,
        drag_point: Vec2,
        falloff: f32,
        target_offset: Vec2,
        v: u128,
    },
}

impl Constraint {
    pub fn relax<Vertlet: GetPosition + SetPosition>(&self, vertlets: &mut HashMap<u128, Vertlet>) {
        match self {
            Constraint::LockPosition { position, v } => {
                let v_verlet = vertlets.get_mut(&v).unwrap();
                v_verlet.set_position(*position);
            }
            Constraint::UnidirectionalDistance {
                v_to_change,
                v_to_get_position_from,
                distance,
            } => {
                let v_to = vertlets.get(&v_to_change).unwrap();
                let v_from = vertlets.get(v_to_get_position_from).unwrap();
                let direction = (v_from.get_position() - v_to.get_position()).normalize_or_zero();
                let delta_d = (v_from.get_position() - v_to.get_position()).length() - distance;
                let v_to = vertlets.get_mut(&v_to_change).unwrap();
                v_to.set_position(v_to.get_position() + (Vec2::splat(delta_d) * direction));
            }
            Constraint::BidirectionalDistance { v1, v2, distance } => {
                let v1_verlet = vertlets.get(&v1).unwrap();
                let v2_vertlet = vertlets.get(&v2).unwrap();
                let direction =
                    (v2_vertlet.get_position() - v1_verlet.get_position()).normalize_or_zero();
                let delta_d =
                    (v2_vertlet.get_position() - v1_verlet.get_position()).length() - distance;
                {
                    let v1_vertlet = vertlets.get_mut(&v1).unwrap();
                    v1_vertlet.set_position(
                        v1_vertlet.get_position()
                            + (Vec2::splat(delta_d) * direction * Vec2::splat(0.5)),
                    );
                };
                {
                    let v2_vertlet = vertlets.get_mut(&v2).unwrap();
                    v2_vertlet.set_position(
                        v2_vertlet.get_position()
                            - (Vec2::splat(delta_d) * direction * Vec2::splat(0.5)),
                    );
                }
            }
            Constraint::InteractionDragging {
                position,
                drag_point,
                falloff,
                target_offset,
                v,
            } => {
                let v_verlet = vertlets.get_mut(&v).unwrap();
                let drag_delta = v_verlet.get_position() - *drag_point;
                let mag_squared = f32::max(drag_delta.length_squared(), 0.001);
                let weight = f32::max(f32::min(falloff / mag_squared, 2.0), 0.0);
                let lerp_end = *position + *target_offset;
                v_verlet.set_position(v_verlet.get_position().lerp(lerp_end, weight));
            }
        };
    }
}
