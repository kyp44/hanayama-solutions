use crate::{
    MARGIN, MAZE_SIZE, NUBS_DISTANCE, SHOE_REAR_DISTANCE,
    controls::{Action, Move, Origin, Rotate, Translate},
};
use derive_new::new;
use macroquad::prelude::*;
use std::f32::consts::PI;

const RADIUS_DELTA: f32 = 1.0;
const ANGLE_DELTA: f32 = 0.75f32.to_radians();

#[derive(Clone, Copy, Default, Debug, new)]
struct Polar {
    /// Radius in pixels
    pub radius: f32,
    /// Angle in radians from -2π to 2π
    pub angle: f32,
}
impl Polar {
    pub fn to_cartesian(self) -> Vec2 {
        polar_to_cartesian(self.radius, self.angle)
    }

    pub fn from_cartesian(v: Vec2) -> Self {
        let p = cartesian_to_polar(v);

        Self::new(p.x, p.y)
    }

    fn bound(&mut self) {
        const BOUND: f32 = MAZE_SIZE / 2. + MARGIN;

        self.radius = self.radius.clamp(0., BOUND);
        self.angle %= 2. * PI;
    }
}

trait Apply {
    fn apply(&self, main: &mut Polar, other: &mut Polar);
}
impl Apply for Translate {
    fn apply(&self, main: &mut Polar, other: &mut Polar) {
        let dv = other.to_cartesian() - main.to_cartesian();

        match self {
            Translate::RadialIn => main.radius -= RADIUS_DELTA,
            Translate::RadialOut => main.radius += RADIUS_DELTA,
            Translate::AngularC => main.angle += ANGLE_DELTA,
            Translate::AngularCC => main.angle -= ANGLE_DELTA,
        }

        // Ensure that the rear is within bounds
        main.bound();

        let old_other = other.to_cartesian();

        // Update the tip position by keeping its radius the same but adjusting the angle
        let x = (main.radius.powi(2) + other.radius.powi(2) - NUBS_DISTANCE.powi(2))
            / (2. * main.radius * other.radius);

        if x.is_nan() || !(-1. ..=1.).contains(&x) {
            // It is not possible to maintain the nub distance while not changing the tip radius.
            // So just move it along with the rear
            *other = Polar::from_cartesian(main.to_cartesian() + dv);
        } else {
            other.angle = main.angle - x.acos();

            if (other.to_cartesian() - old_other).length() > 15. {
                // The other was moved in a discontinuous way.
                // Most likely this is because the angle difference flipped about
                // 180 degrees due to the angle difference being in the lower half
                // plane, but the acos is an angle difference in the upper half
                // plane.
                other.angle = main.angle + x.acos();
            }
        }
    }
}
impl Apply for Rotate {
    fn apply(&self, main: &mut Polar, other: &mut Polar) {
        let main = main.to_cartesian();

        // Rotate the tip around the rear
        let mut dvp = Polar::from_cartesian(other.to_cartesian() - main);

        match self {
            Rotate::AngularC => dvp.angle += ANGLE_DELTA,
            Rotate::AngularCC => dvp.angle -= ANGLE_DELTA,
        }

        *other = Polar::from_cartesian(main + dvp.to_cartesian());
    }
}
impl Apply for Move {
    fn apply(&self, main: &mut Polar, other: &mut Polar) {
        match self {
            Move::Translation(t) => t.apply(main, other),
            Move::Rotation(r) => r.apply(main, other),
        }
    }
}

pub struct Undo<S> {
    stack: Vec<S>,
}
impl<S> Undo<S> {
    pub fn new(initial: S) -> Self {
        Self {
            stack: vec![initial],
        }
    }

    pub fn new_state(&mut self, state: S) {
        self.stack.push(state);
    }

    pub fn undo(&mut self) -> Option<S> {
        (self.stack.len() > 1).then(|| self.stack.pop().unwrap())
    }

    pub fn current(&self) -> &S {
        self.stack.last().unwrap()
    }
}

#[derive(Clone)]
pub struct State {
    rear_position: Polar,
    tip_position: Polar,
}
impl Default for State {
    fn default() -> Self {
        Self {
            rear_position: Polar::new(NUBS_DISTANCE / 2., PI),
            tip_position: Polar::new(NUBS_DISTANCE / 2., 0.),
        }
    }
}
impl State {
    // Relative to the center of the maze
    pub fn rear_nub_position(&self) -> Vec2 {
        self.rear_position.to_cartesian()
    }

    // Relative to the center of the maze
    pub fn tip_nub_position(&self) -> Vec2 {
        self.tip_position.to_cartesian()
    }

    pub fn shoe_position(&self) -> Vec2 {
        let rear = self.rear_nub_position();
        rear + SHOE_REAR_DISTANCE * (rear - self.tip_nub_position()).normalize()
    }

    /// Returns whether to quit or not
    pub fn apply_action(&self, action: Action) -> Self {
        match action {
            Action::Reset => Self::default(),
            Action::Move { origin, muv } => {
                let mut state = self.clone();

                match origin {
                    Origin::RearNub => muv.apply(&mut state.rear_position, &mut state.tip_position),
                    Origin::TipNub => muv.apply(&mut state.tip_position, &mut state.rear_position),
                }

                state
            }
            _ => panic!(),
        }
    }
}
