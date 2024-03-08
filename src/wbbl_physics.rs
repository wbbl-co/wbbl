use glam::Vec2;
use web_sys::CanvasRenderingContext2d;

const BOX_X_COUNT: usize = 4;
const BOX_Y_COUNT: usize = 4;
const BOX_CORNER_RADIUS: f32 = 20.0;
const BOX_SPRING_DAMPING_COEFFICIENT: f32 = 6.0;
const BOX_SPRING_SPRING_COEFFICIENT: f32 = 75.0;
const BOX_INTERACTION_DRAGGING_CONSTRAINT_FALLOFF: f32 = 200.0;
const UPDATE_ITERATIONS: u32 = 2;
const CONSTRAINT_ITERATIONS: u32 = 4;
const ROPE_GRAVITY: f32 = 200.0;
const ROPE_VERTLET_COUNT: usize = 8;
const ROPE_MIN_TARGET_DISTANCE: f32 = 50.0;
const ROPE_SLACK: f32 = 1.1;

pub trait GetPosition {
    fn get_position(&self) -> Vec2;
}

pub trait SetPosition {
    fn set_position(&mut self, position: Vec2);
}

pub trait Vertlet {
    fn update(&mut self, delta_time: f32, delta_time_squared: f32);
}

#[derive(Clone, Default)]
pub struct VelocityVertlet {
    pub acceleration: Vec2,
    pub position: Vec2,
    pub velocity: Vec2,
    pub new_acceleration: Vec2,
}

impl VelocityVertlet {
    pub fn gather_forces(&self, forces: &Vec<Force>) -> Vec2 {
        forces.iter().fold(Vec2::ZERO, |result, force| {
            result + force.get_accelleration(self)
        })
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

#[derive(Clone)]
pub enum Force {
    Damper(f32),
    Spring {
        coefficient: f32,
        target_position: Vec2,
    },
}

impl Force {
    pub fn get_accelleration(&self, v: &VelocityVertlet) -> Vec2 {
        match self {
            Force::Damper(coefficient) => Vec2::splat(-coefficient) * v.velocity,
            Force::Spring {
                coefficient,
                target_position,
            } => {
                let delta = *target_position - v.position;
                Vec2::splat(-coefficient * delta.length()) * (delta.normalize_or_zero())
            }
        }
    }
}

pub enum Constraint {
    LockPosition {
        position: Vec2,
        v: usize,
    },
    UnidirectionalDistance {
        v_to_change: usize,
        v_to_get_position_from: usize,
        distance: f32,
    },
    BidirectionalDistance {
        v1: usize,
        v2: usize,
        distance: f32,
    },
    InteractionDragging {
        target_position: Vec2,
        drag_point: Vec2,
        falloff: f32,
        v: usize,
    },
}

impl Constraint {
    pub fn relax<Vertlet: GetPosition + SetPosition>(&self, vertlets: &mut Vec<Vertlet>) {
        match self {
            Constraint::LockPosition { position, v } => {
                let v_verlet = &mut vertlets[*v];
                v_verlet.set_position(*position);
            }
            Constraint::UnidirectionalDistance {
                v_to_change,
                v_to_get_position_from,
                distance,
            } => {
                let v_to = &vertlets[*v_to_change];
                let v_from = &vertlets[*v_to_get_position_from];
                let direction = (v_from.get_position() - v_to.get_position()).normalize_or_zero();
                let delta_d = (v_from.get_position() - v_to.get_position()).length() - distance;
                let v_to = &mut vertlets[*v_to_change];
                v_to.set_position(v_to.get_position() + (Vec2::splat(delta_d) * direction));
            }
            Constraint::BidirectionalDistance { v1, v2, distance } => {
                let v1_verlet = &vertlets[*v1];
                let v2_vertlet = &vertlets[*v2];
                let direction =
                    (v2_vertlet.get_position() - v1_verlet.get_position()).normalize_or_zero();
                let delta_d =
                    (v2_vertlet.get_position() - v1_verlet.get_position()).length() - distance;
                {
                    let v1_vertlet = &mut vertlets[*v1];
                    v1_vertlet.set_position(
                        v1_vertlet.get_position()
                            + (Vec2::splat(delta_d) * direction * Vec2::splat(0.5)),
                    );
                };
                {
                    let v2_vertlet = &mut vertlets[*v2];
                    v2_vertlet.set_position(
                        v2_vertlet.get_position()
                            - (Vec2::splat(delta_d) * direction * Vec2::splat(0.5)),
                    );
                }
            }
            Constraint::InteractionDragging {
                target_position,
                drag_point,
                falloff,
                v,
            } => {
                let v_verlet = &mut vertlets[*v];
                let drag_delta = v_verlet.get_position() - *drag_point;
                let mag_squared = f32::max(drag_delta.length_squared(), 0.001);
                let weight = f32::max(f32::min(falloff / mag_squared, 2.0), 0.0);
                v_verlet.set_position(v_verlet.get_position().lerp(*target_position, weight));
            }
        };
    }
}

pub struct Box {
    vertlets: Vec<VelocityVertlet>,
    top: Vec<usize>,
    bottom: Vec<usize>,
    left: Vec<usize>,
    right: Vec<usize>,
}

impl Box {
    pub fn new(initial_position_top_left: [f32; 2], box_size: [f32; 2]) -> Box {
        let initial_position_top_left = Vec2::from_array(initial_position_top_left);
        let box_size = Vec2::from_array(box_size);
        let mut result = Box {
            vertlets: Vec::new(),
            top: Vec::new(),
            bottom: Vec::new(),
            left: Vec::new(),
            right: Vec::new(),
        };

        let x_stride = Vec2::new(box_size.x / (BOX_X_COUNT as f32), 0.0);
        let y_stride = Vec2::new(0.0, box_size.x / (BOX_Y_COUNT as f32));

        // TOP
        for i in 0..BOX_X_COUNT {
            let offset = x_stride * Vec2::splat(i as f32);
            let intial_position = offset + initial_position_top_left;
            result.top.push(result.vertlets.len());
            result.vertlets.push(VelocityVertlet {
                acceleration: Vec2::ZERO,
                position: intial_position,
                velocity: Vec2::ZERO,
                new_acceleration: Vec2::ZERO,
            });
        }
        // BOTTOM
        for i in (0..BOX_X_COUNT).rev() {
            let offset = (x_stride * Vec2::splat(i as f32)) + box_size * Vec2::new(0.0, 1.0);
            let intial_position = offset + initial_position_top_left;
            result.bottom.push(result.vertlets.len());
            result.vertlets.push(VelocityVertlet {
                acceleration: Vec2::ZERO,
                position: intial_position,
                velocity: Vec2::ZERO,
                new_acceleration: Vec2::ZERO,
            });
        }
        // LEFT
        for i in (1..BOX_Y_COUNT).rev() {
            let offset = y_stride * Vec2::splat(i as f32);
            let intial_position = offset + initial_position_top_left;
            result.left.push(result.vertlets.len());
            result.vertlets.push(VelocityVertlet {
                acceleration: Vec2::ZERO,
                position: intial_position,
                velocity: Vec2::ZERO,
                new_acceleration: Vec2::ZERO,
            });
        }
        // RIGHT
        for i in 1..BOX_Y_COUNT {
            let offset = (y_stride * Vec2::splat(i as f32)) + box_size * Vec2::new(1.0, 0.0);
            let intial_position = offset + initial_position_top_left;
            result.right.push(result.vertlets.len());
            result.vertlets.push(VelocityVertlet {
                acceleration: Vec2::ZERO,
                position: intial_position,
                velocity: Vec2::ZERO,
                new_acceleration: Vec2::ZERO,
            });
        }

        result
    }

    pub fn update(
        &mut self,
        position_top_left: [f32; 2],
        box_size: [f32; 2],
        delta_time: f32,
        drag_point: Option<[f32; 2]>,
    ) {
        let position_top_left = Vec2::from_array(position_top_left);
        let box_size = Vec2::from_array(box_size);

        let x_stride = Vec2::new(box_size.x / (BOX_X_COUNT as f32), 0.0);
        let y_stride = Vec2::new(0.0, box_size.x / (BOX_Y_COUNT as f32));

        let mut forces: Vec<Vec<Force>> = Vec::new();
        let mut constraints: Vec<Constraint> = Vec::new();

        let damping_force = Force::Damper(BOX_SPRING_DAMPING_COEFFICIENT);

        // Initilize default forces for each vertlet
        for _ in 0..self.vertlets.len() {
            let mut force_for_verlet = Vec::new();
            force_for_verlet.push(damping_force.clone());
            forces.push(force_for_verlet);
        }

        let drag_point = drag_point.map(Vec2::from_array);

        // TOP
        for i in 0..BOX_X_COUNT {
            let offset = x_stride * Vec2::splat(i as f32);
            let target_position = offset + position_top_left;
            let index = self.top[i];
            forces[index].push(Force::Spring {
                coefficient: BOX_SPRING_SPRING_COEFFICIENT,
                target_position,
            });
            if let Some(drag_point) = drag_point {
                constraints.push(Constraint::InteractionDragging {
                    target_position,
                    drag_point,
                    falloff: BOX_INTERACTION_DRAGGING_CONSTRAINT_FALLOFF,
                    v: index,
                });
            }
        }

        // BOTTOM
        for i in 0..BOX_X_COUNT {
            let offset = (x_stride * Vec2::splat(i as f32)) + box_size * Vec2::new(0.0, 1.0);
            let target_position = offset + position_top_left;
            // Bottom's order is reversed for efficient drawing
            let index = self.bottom[self.bottom.len() - 1 - i];
            forces[index].push(Force::Spring {
                coefficient: BOX_SPRING_SPRING_COEFFICIENT,
                target_position,
            });
            if let Some(drag_point) = drag_point {
                constraints.push(Constraint::InteractionDragging {
                    target_position,
                    drag_point,
                    falloff: BOX_INTERACTION_DRAGGING_CONSTRAINT_FALLOFF,
                    v: index,
                });
            }
        }

        // LEFT
        for i in 1..BOX_Y_COUNT {
            let offset = y_stride * Vec2::splat(i as f32);
            let target_position = offset + position_top_left;
            // Lefts order is reversed for efficient drawing
            let index = self.left[self.left.len() - i];
            forces[index].push(Force::Spring {
                coefficient: BOX_SPRING_SPRING_COEFFICIENT,
                target_position,
            });
            if let Some(drag_point) = drag_point {
                constraints.push(Constraint::InteractionDragging {
                    target_position,
                    drag_point,
                    falloff: BOX_INTERACTION_DRAGGING_CONSTRAINT_FALLOFF,
                    v: index,
                });
            }
        }

        // RIGHT
        for i in 1..BOX_Y_COUNT {
            let offset = (y_stride * Vec2::splat(i as f32)) + box_size * Vec2::new(1.0, 0.0);
            let target_position = offset + position_top_left;
            let index = self.right[i - 1];
            forces[index].push(Force::Spring {
                coefficient: BOX_SPRING_SPRING_COEFFICIENT,
                target_position,
            });
            if let Some(drag_point) = drag_point {
                constraints.push(Constraint::InteractionDragging {
                    target_position,
                    drag_point,
                    falloff: BOX_INTERACTION_DRAGGING_CONSTRAINT_FALLOFF,
                    v: index,
                });
            }
        }

        let delta_time = delta_time / (UPDATE_ITERATIONS as f32);
        let delta_time_squared = delta_time * delta_time;

        for _ in 0..UPDATE_ITERATIONS {
            for i in 0..self.vertlets.len() {
                let vertlet = &mut self.vertlets[i];
                let force_for_verlet = &forces[i];
                vertlet.new_acceleration = vertlet.gather_forces(force_for_verlet);
            }

            for vertlet in self.vertlets.iter_mut() {
                vertlet.update(delta_time, delta_time_squared);
            }
        }

        for _ in 0..CONSTRAINT_ITERATIONS {
            for constraint in constraints.iter() {
                constraint.relax(&mut self.vertlets);
            }
        }
    }

    fn draw_corner(
        context: &CanvasRenderingContext2d,
        z: Vec2,
        a: Vec2,
        b: Vec2,
        c: Vec2,
        canvas_position: Vec2,
    ) {
        let delta_ab = b - a;
        let delta_cb = c - b;

        let p1 = (b - delta_ab.normalize() * BOX_CORNER_RADIUS) - canvas_position;
        let p2 = (b + delta_cb.normalize() * BOX_CORNER_RADIUS) - canvas_position;
        let a = a - canvas_position;
        let b = b - canvas_position;
        let z = z - canvas_position;
        context.bezier_curve_to(
            z.x as f64,
            z.y as f64,
            a.x as f64,
            a.y as f64,
            p1.x as f64,
            p1.y as f64,
        );

        context.quadratic_curve_to(b.x as f64, b.y as f64, p2.x as f64, p2.y as f64);
    }

    fn get_pos(&self, index: usize) -> Vec2 {
        self.vertlets[index].position
    }

    pub fn draw(&self, context: &CanvasRenderingContext2d, canvas_position: [f32; 2]) {
        let canvas_position = Vec2::from_array(canvas_position);
        context.begin_path();

        let start_point = self.get_pos(self.top[0])
            + (self.get_pos(self.top[1]) - self.get_pos(self.top[0])).normalize()
                * BOX_CORNER_RADIUS
            - canvas_position;
        context.move_to(start_point.x as f64, start_point.y as f64);
        Box::draw_corner(
            context,
            self.get_pos(self.top[self.top.len() - 3]),
            self.get_pos(self.top[self.top.len() - 2]),
            self.get_pos(self.top[self.top.len() - 1]),
            self.get_pos(self.right[0]),
            canvas_position,
        );

        Box::draw_corner(
            context,
            self.get_pos(self.right[self.right.len() - 2]),
            self.get_pos(self.right[self.right.len() - 1]),
            self.get_pos(self.bottom[0]),
            self.get_pos(self.bottom[1]),
            canvas_position,
        );

        Box::draw_corner(
            context,
            self.get_pos(self.bottom[self.bottom.len() - 3]),
            self.get_pos(self.bottom[self.bottom.len() - 2]),
            self.get_pos(self.bottom[self.bottom.len() - 1]),
            self.get_pos(self.left[0]),
            canvas_position,
        );

        Box::draw_corner(
            context,
            self.get_pos(self.left[self.left.len() - 2]),
            self.get_pos(self.left[self.left.len() - 1]),
            self.get_pos(self.top[0]),
            self.get_pos(self.top[1]),
            canvas_position,
        );
        context.close_path();
    }
}

pub struct Rope {
    vertlets: Vec<PositionVertlet>,
}

impl Rope {
    pub fn new(start: [f32; 2], end: [f32; 2]) -> Rope {
        let start = Vec2::from_array(start);
        let end = Vec2::from_array(end);

        let mut result = Rope {
            vertlets: Vec::new(),
        };

        for i in 0..ROPE_VERTLET_COUNT {
            let position = start.lerp(end, i as f32 / ((ROPE_VERTLET_COUNT - 1) as f32));
            result.vertlets.push(PositionVertlet {
                acceleration: Vec2::ZERO,
                previous_position: position.clone(),
                position: position.clone(),
                new_acceleration: Vec2::ZERO,
            });
        }

        result
    }

    pub fn update(&mut self, start: [f32; 2], end: [f32; 2], delta_time: f32) {
        let start = Vec2::from_array(start);
        let end = Vec2::from_array(end);

        let delta_time = delta_time / (UPDATE_ITERATIONS as f32);
        let delta_time_squared = delta_time * delta_time;

        for _ in 0..UPDATE_ITERATIONS {
            for i in 0..self.vertlets.len() {
                let vertlet = &mut self.vertlets[i];
                vertlet.new_acceleration = Vec2::new(0.0, ROPE_GRAVITY);
            }

            for vertlet in self.vertlets.iter_mut() {
                vertlet.update(delta_time, delta_time_squared);
            }
        }

        let mut constraints: Vec<Constraint> = Vec::new();
        let last_index = ROPE_VERTLET_COUNT - 1;
        let second_last_index = last_index - 1;

        let target_distance = f32::max(ROPE_MIN_TARGET_DISTANCE, (start - end).length())
            * (ROPE_SLACK / (ROPE_VERTLET_COUNT as f32));

        for i in 0..ROPE_VERTLET_COUNT {
            let target_position = start.lerp(end, i as f32 / ((ROPE_VERTLET_COUNT - 1) as f32));
            if i == 0 || i == last_index {
                constraints.push(Constraint::LockPosition {
                    position: target_position,
                    v: i,
                });
            } else if i == 1 {
                constraints.push(Constraint::UnidirectionalDistance {
                    v_to_change: i,
                    v_to_get_position_from: i - 1,
                    distance: target_distance,
                });
            } else if i == second_last_index {
                constraints.push(Constraint::BidirectionalDistance {
                    v1: i,
                    v2: i - 1,
                    distance: target_distance,
                });
                constraints.push(Constraint::UnidirectionalDistance {
                    v_to_change: i,
                    v_to_get_position_from: i + 1,
                    distance: target_distance,
                });
            } else {
                constraints.push(Constraint::BidirectionalDistance {
                    v1: i,
                    v2: i - 1,
                    distance: target_distance,
                });
            }
        }

        for _ in 0..CONSTRAINT_ITERATIONS {
            for constraint in constraints.iter() {
                constraint.relax(&mut self.vertlets);
            }
        }
    }

    pub fn draw(&self, context: &CanvasRenderingContext2d, canvas_position: [f32; 2]) {
        let canvas_position = Vec2::from_array(canvas_position);

        context.begin_path();
        context.move_to(
            (self.vertlets[0].position.x - canvas_position.x) as f64,
            (self.vertlets[0].position.y - canvas_position.y) as f64,
        );
        for i in 1..ROPE_VERTLET_COUNT {
            context.line_to(
                (self.vertlets[i].position.x - canvas_position.x) as f64,
                (self.vertlets[i].position.y - canvas_position.y) as f64,
            );
        }
    }
}
