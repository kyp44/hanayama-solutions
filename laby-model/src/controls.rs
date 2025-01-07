use derive_more::derive::Display;
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum Translate {
    RadialIn,
    RadialOut,
    AngularC,
    AngularCC,
}
impl From<ArrowKey> for Translate {
    fn from(value: ArrowKey) -> Self {
        match value {
            ArrowKey::Up => Self::RadialOut,
            ArrowKey::Down => Self::RadialIn,
            ArrowKey::Left => Self::AngularCC,
            ArrowKey::Right => Self::AngularC,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Rotate {
    AngularC,
    AngularCC,
}
impl From<ArrowKey> for Rotate {
    fn from(value: ArrowKey) -> Self {
        match value {
            ArrowKey::Up => Self::AngularCC,
            ArrowKey::Down => Self::AngularC,
            ArrowKey::Left => Self::AngularCC,
            ArrowKey::Right => Self::AngularC,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Move {
    Translation(Translate),
    Rotation(Rotate),
}

#[derive(Clone, Copy, Debug, Default, Display)]
pub enum Origin {
    #[default]
    #[display("Rear")]
    RearNub,
    #[display("Tip")]
    TipNub,
}
impl Origin {
    pub fn color(&self) -> Color {
        match self {
            Origin::RearNub => RED,
            Origin::TipNub => BLUE,
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Origin::RearNub => Self::TipNub,
            Origin::TipNub => Self::RearNub,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Action {
    Quit,
    Reset,
    Move { origin: Origin, muv: Move },
    Undo,
}

#[derive(Clone, Copy, Debug, Default, Display)]
pub enum Mode {
    #[default]
    Translate,
    Rotate,
}
impl Mode {
    pub fn next(&self) -> Self {
        match self {
            Mode::Translate => Self::Rotate,
            Mode::Rotate => Self::Translate,
        }
    }

    pub fn muv(&self, arrow_key: ArrowKey) -> Move {
        match self {
            Mode::Translate => Move::Translation(arrow_key.into()),
            Mode::Rotate => Move::Rotation(arrow_key.into()),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ArrowKey {
    Up,
    Down,
    Left,
    Right,
}
impl ArrowKey {
    pub fn check_down() -> Option<Self> {
        let keys_down = get_keys_down();

        if keys_down.contains(&KeyCode::Up) {
            Some(Self::Up)
        } else if keys_down.contains(&KeyCode::Down) {
            Some(Self::Down)
        } else if keys_down.contains(&KeyCode::Left) {
            Some(Self::Left)
        } else if keys_down.contains(&KeyCode::Right) {
            Some(Self::Right)
        } else {
            None
        }
    }
}

#[derive(Default)]
pub struct Controller {
    origin: Origin,
    mode: Mode,
}
impl Controller {
    pub fn current_origin(&self) -> Origin {
        self.origin
    }

    pub fn current_mode(&self) -> Mode {
        self.mode
    }

    pub fn check_for_action(&mut self) -> Option<Action> {
        if is_key_pressed(KeyCode::Q) || is_key_pressed(KeyCode::Escape) {
            return Some(Action::Quit);
        }

        if is_key_pressed(KeyCode::R) {
            // Reset controls
            *self = Self::default();

            return Some(Action::Reset);
        }

        if is_key_down(KeyCode::Backspace) {
            return Some(Action::Undo);
        }

        if is_key_pressed(KeyCode::Tab) {
            self.origin = self.origin.next();
        }

        if is_key_pressed(KeyCode::Space) {
            self.mode = self.mode.next();
        }

        ArrowKey::check_down().map(|a| Action::Move {
            origin: self.origin,
            muv: self.mode.muv(a),
        })
    }
}
