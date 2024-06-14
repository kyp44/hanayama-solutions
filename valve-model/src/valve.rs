use bare_metal_modulo::{MNum, ModNumC};
use euclid::{Point2D, Vector2D};
use itertools::Itertools;
use maplit::hashset;
use non_empty_collections::NonEmptyIndexSet;
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    marker::PhantomData,
    ops::{AddAssign, SubAssign},
    sync::LazyLock,
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use undo::Edit;

use crate::Absolute;

#[derive(Debug, PartialEq, Eq)]
pub struct ModPoint<U> {
    pub x: ModNumC<i32, 6>,
    pub y: i32,
    _phantom: PhantomData<U>,
}
impl<U> ModPoint<U> {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x: ModNumC::new(x),
            y,
            _phantom: PhantomData,
        }
    }

    pub fn cast_unit<V>(self) -> ModPoint<V> {
        ModPoint {
            x: self.x,
            y: self.y,
            _phantom: PhantomData,
        }
    }
}
impl<U> Clone for ModPoint<U> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<U> Copy for ModPoint<U> {}
impl<U> From<ModPoint<U>> for Point2D<i32, U> {
    fn from(value: ModPoint<U>) -> Self {
        Self::new(value.x.a(), value.y)
    }
}
impl<U> std::ops::Add for ModPoint<U> {
    type Output = ModPoint<U>;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            _phantom: PhantomData,
        }
    }
}
impl<U> AddAssign for ModPoint<U> {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}
impl<U> std::ops::Sub for ModPoint<U> {
    type Output = ModPoint<U>;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            _phantom: PhantomData,
        }
    }
}
impl<U> SubAssign for ModPoint<U> {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

pub struct Inside;
pub type InsidePoint = Point2D<i32, Inside>;

pub struct Outside;
pub type OutsidePoint = Point2D<i32, Outside>;

#[derive(Default)]
pub struct HanayamaHalf;
impl HanayamaHalf {
    pub fn inside_points(&self) -> &'static HashSet<InsidePoint> {
        static HANAYAMA_HALF_INSIDE_POINTS: LazyLock<HashSet<InsidePoint>> = LazyLock::new(|| {
            hashset! {
                InsidePoint::new(0, 0),
                InsidePoint::new(1, 0),
                InsidePoint::new(2, 0),
                InsidePoint::new(0, 1),
                InsidePoint::new(0, 2),
                InsidePoint::new(2, 3),
                InsidePoint::new(0, 4),
                InsidePoint::new(1, 4),
                InsidePoint::new(2, 4),
            }
        });

        &HANAYAMA_HALF_INSIDE_POINTS
    }

    pub fn outside_points(&self) -> &'static HashSet<OutsidePoint> {
        static HANAYAMA_HALF_OUTSIDE_POINTS: LazyLock<HashSet<OutsidePoint>> =
            LazyLock::new(|| {
                hashset! {
                    OutsidePoint::new(3, 0),
                    OutsidePoint::new(4, 0),
                    OutsidePoint::new(5, 0),
                    OutsidePoint::new(5, 2),
                    OutsidePoint::new(3, 4),
                    OutsidePoint::new(4, 4),
                    OutsidePoint::new(5, 4),
                }
            });

        &HANAYAMA_HALF_OUTSIDE_POINTS
    }
}

#[derive(Default)]
pub struct ValveHalf {
    y: i32,
}
impl ValveHalf {
    pub fn inside_points(&self, shift: Option<i32>) -> HashSet<InsidePoint> {
        static VALVE_HALF_INSIDE_POINTS: LazyLock<HashSet<Point2D<i32, ValveHalf>>> =
            LazyLock::new(|| {
                hashset! {
                    Point2D::new(0, 0),
                    Point2D::new(1, 0),
                    Point2D::new(2, 0),
                    Point2D::new(2, 2),
                    Point2D::new(0, 4),
                    Point2D::new(1, 4),
                    Point2D::new(2, 4),
                }
            });

        let shift = shift.unwrap_or(0);
        VALVE_HALF_INSIDE_POINTS
            .iter()
            .map(|p| p.cast_unit() + Vector2D::new(3, self.y + shift))
            .collect()
    }

    pub fn outside_points(&self, shift: Option<i32>) -> HashSet<OutsidePoint> {
        static VALVE_HALF_OUTSIDE_POINTS: LazyLock<HashSet<Point2D<i32, ValveHalf>>> =
            LazyLock::new(|| {
                hashset! {
                    Point2D::new(0, 0),
                    Point2D::new(1, 0),
                    Point2D::new(2, 0),
                    Point2D::new(0, 2),
                    Point2D::new(1, 2),
                    Point2D::new(0, 4),
                    Point2D::new(1, 4),
                    Point2D::new(2, 4),
                }
            });

        let shift = shift.unwrap_or(0);
        VALVE_HALF_OUTSIDE_POINTS
            .iter()
            .map(|p| p.cast_unit() + Vector2D::new(0, self.y + shift))
            .collect()
    }

    pub fn shift(&mut self, dy: i32) {
        self.y += dy;
    }
}

pub trait Ring: Sized {
    type Coordinates;

    fn position(&self) -> ModPoint<Self::Coordinates>;
    fn points_local(&self) -> &[ModPoint<Self>];
    fn shift(&mut self, vector: Vector2D<i32, Self::Coordinates>);

    fn points(
        &self,
        shift: Option<Vector2D<i32, Self::Coordinates>>,
    ) -> impl Iterator<Item = Point2D<i32, Self::Coordinates>> {
        let shift = match shift {
            Some(v) => ModPoint::new(v.x, v.y),
            None => ModPoint::new(0, 0),
        };

        self.points_local()
            .iter()
            .map(move |p| (self.position() + p.cast_unit() + shift).into())
    }

    fn can_move(
        &self,
        hanayama_points: &HashSet<Point2D<i32, Self::Coordinates>>,
        valve_points: &HashSet<Point2D<i32, Self::Coordinates>>,
        direction: Vector2D<i32, Self::Coordinates>,
    ) -> bool {
        self.points(Some(direction))
            .all(|p| !hanayama_points.contains(&p) && !valve_points.contains(&p))
    }
}

pub struct InnerRing {
    position: ModPoint<Inside>,
}
impl Default for InnerRing {
    fn default() -> Self {
        Self {
            position: ModPoint::new(4, 1),
        }
    }
}
impl Ring for InnerRing {
    type Coordinates = Inside;

    fn position(&self) -> ModPoint<Self::Coordinates> {
        self.position
    }

    fn points_local(&self) -> &[ModPoint<Self>] {
        static INNER_RING_POINTS: LazyLock<[ModPoint<InnerRing>; 4]> = LazyLock::new(|| {
            [
                ModPoint::new(0, 1),
                ModPoint::new(1, 2),
                ModPoint::new(2, 2),
                ModPoint::new(3, 2),
            ]
        });

        INNER_RING_POINTS.as_ref()
    }

    fn shift(&mut self, vector: Vector2D<i32, Self::Coordinates>) {
        self.position += ModPoint::new(vector.x, vector.y);
    }
}

pub struct OuterRing {
    position: ModPoint<Outside>,
}
impl Default for OuterRing {
    fn default() -> Self {
        Self {
            position: ModPoint::new(0, 1),
        }
    }
}
impl Ring for OuterRing {
    type Coordinates = Outside;

    fn position(&self) -> ModPoint<Self::Coordinates> {
        self.position
    }

    fn points_local(&self) -> &[ModPoint<Self>] {
        static OUTER_RING_POINTS: LazyLock<[ModPoint<OuterRing>; 3]> = LazyLock::new(|| {
            [
                ModPoint::new(0, 0),
                ModPoint::new(3, 2),
                ModPoint::new(5, 2),
            ]
        });

        OUTER_RING_POINTS.as_ref()
    }

    fn shift(&mut self, vector: Vector2D<i32, Self::Coordinates>) {
        self.position += ModPoint::new(vector.x, vector.y);
    }
}

#[derive(Debug, EnumIter, Clone, Copy)]
pub enum RingDirection {
    Left,
    Right,
}
impl std::ops::Neg for RingDirection {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            RingDirection::Left => Self::Right,
            RingDirection::Right => Self::Left,
        }
    }
}
impl RingDirection {
    pub fn vector<U>(&self) -> Vector2D<i32, U> {
        Vector2D::new(
            match self {
                RingDirection::Left => -1,
                RingDirection::Right => 1,
            },
            0,
        )
    }
}

#[derive(Debug, EnumIter, Clone, Copy)]
pub enum VerticalDirection {
    Up,
    Down,
}
impl std::ops::Neg for VerticalDirection {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            VerticalDirection::Up => Self::Down,
            VerticalDirection::Down => Self::Up,
        }
    }
}
impl VerticalDirection {
    pub fn vector<U>(&self) -> Vector2D<i32, U> {
        Vector2D::new(
            0,
            match self {
                VerticalDirection::Up => -1,
                VerticalDirection::Down => 1,
            },
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum VerticalPieces {
    InnerRing,
    OuterRing,
    ValveHalf,
}

pub trait NonEmptyIndexSetExt: Sized {
    fn vertical_piece_iter() -> impl Iterator<Item = Self>;

    fn vertical_displacements(&self, dir: VerticalDirection) -> HashMap<VerticalPieces, i32>;
}

impl NonEmptyIndexSetExt for NonEmptyIndexSet<VerticalPieces> {
    fn vertical_piece_iter() -> impl Iterator<Item = Self> {
        (1..=3).flat_map(|n| {
            VerticalPieces::iter()
                .combinations(n)
                .map(|v| NonEmptyIndexSet::from_iterator(v).unwrap())
        })
    }

    fn vertical_displacements(&self, dir: VerticalDirection) -> HashMap<VerticalPieces, i32> {
        let dy = dir.vector::<Absolute>().y;

        VerticalPieces::iter()
            .map(|vp| (vp, if self.contains(&vp) { dy } else { 0 }))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub enum Move {
    InnerRingRotate(RingDirection),
    OuterRingRotate(RingDirection),
    Vertical {
        direction: VerticalDirection,
        pieces: NonEmptyIndexSet<VerticalPieces>,
    },
}
impl std::ops::Neg for Move {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Move::InnerRingRotate(d) => Self::InnerRingRotate(-d),
            Move::OuterRingRotate(d) => Self::OuterRingRotate(-d),
            Move::Vertical { direction, pieces } => Self::Vertical {
                direction: -direction,
                pieces,
            },
        }
    }
}
impl Edit for Move {
    type Target = Valve;
    type Output = ();

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        target.make_move_unchecked(self);
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        target.make_move_unchecked(&-self.clone());
    }
}
impl Move {
    pub fn iter() -> impl Iterator<Item = Move> {
        RingDirection::iter()
            .map(Self::InnerRingRotate)
            .chain(RingDirection::iter().map(Self::OuterRingRotate))
            .chain(VerticalDirection::iter().flat_map(|d| {
                NonEmptyIndexSet::vertical_piece_iter().map(move |pieces| Self::Vertical {
                    direction: d,
                    pieces,
                })
            }))
    }

    pub fn as_str(&self) -> Cow<'static, str> {
        match self {
            Move::InnerRingRotate(d) => match d {
                RingDirection::Left => "IL",
                RingDirection::Right => "IR",
            }
            .into(),
            Move::OuterRingRotate(d) => match d {
                RingDirection::Left => "OL",
                RingDirection::Right => "OR",
            }
            .into(),
            Move::Vertical { direction, pieces } => format!(
                "{}{{{}}}",
                match direction {
                    VerticalDirection::Up => "U",
                    VerticalDirection::Down => "D",
                },
                pieces
                    .iter()
                    .map(|vp| match vp {
                        VerticalPieces::InnerRing => "I",
                        VerticalPieces::OuterRing => "O",
                        VerticalPieces::ValveHalf => "V",
                    })
                    .join(", ")
            )
            .into(),
        }
    }
}

pub const VALVE_MAX_HALF_Y_DISPLACEMENT: i32 = 4;
pub const VALVE_SECTION_WIDTH: i32 = 6;
pub const VALVE_SECTION_HEIGHT: i32 = 5;

#[derive(Default)]
pub struct Valve {
    pub inner_ring: InnerRing,
    pub outer_ring: OuterRing,
    pub valve_half: ValveHalf,
    pub hanayama_half: HanayamaHalf,
}
impl Valve {
    pub fn can_move(&self, muv: &Move) -> bool {
        match muv {
            Move::InnerRingRotate(dir) => self.inner_ring.can_move(
                self.hanayama_half.inside_points(),
                &self.valve_half.inside_points(None),
                dir.vector(),
            ),
            Move::OuterRingRotate(dir) => self.outer_ring.can_move(
                self.hanayama_half.outside_points(),
                &self.valve_half.outside_points(None),
                dir.vector(),
            ),
            Move::Vertical { direction, pieces } => {
                let displacements = pieces.vertical_displacements(*direction);
                let valve_displacement = Some(displacements[&VerticalPieces::ValveHalf]);

                self.inner_ring.can_move(
                    self.hanayama_half.inside_points(),
                    &self.valve_half.inside_points(valve_displacement),
                    Vector2D::new(0, displacements[&VerticalPieces::InnerRing]),
                ) && self.outer_ring.can_move(
                    self.hanayama_half.outside_points(),
                    &self.valve_half.outside_points(valve_displacement),
                    Vector2D::new(0, displacements[&VerticalPieces::OuterRing]),
                )
            }
        }
    }

    pub fn make_move_unchecked(&mut self, muv: &Move) {
        match muv {
            Move::InnerRingRotate(dir) => {
                self.inner_ring.shift(dir.vector());
            }
            Move::OuterRingRotate(dir) => {
                self.outer_ring.shift(dir.vector());
            }
            Move::Vertical { direction, pieces } => {
                let shift = direction.vector();
                if pieces.contains(&VerticalPieces::InnerRing) {
                    self.inner_ring.shift(shift)
                }
                if pieces.contains(&VerticalPieces::OuterRing) {
                    self.outer_ring.shift(shift.cast_unit())
                }
                if pieces.contains(&VerticalPieces::ValveHalf) {
                    self.valve_half.shift(shift.y)
                }
            }
        }
    }
}
