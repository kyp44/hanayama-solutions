#![feature(try_blocks)]

mod valve;

use std::sync::LazyLock;

use crate::valve::{
    InsidePoint, OutsidePoint, VALVE_MAX_HALF_Y_DISPLACEMENT, VALVE_SECTION_HEIGHT,
    VALVE_SECTION_WIDTH,
};
use easycurses::{Color, ColorPair, CursorVisibility, EasyCurses, Input, InputMode};
use euclid::{Point2D, Translation2D};
use itertools::{iproduct, Itertools};
use non_empty_collections::NonEmptyIndexSet;
use thiserror::Error;
use undo::Record;
use valve::{
    Inside, Move, NonEmptyIndexSetExt, Outside, Ring, RingDirection, Valve, VerticalDirection,
};

struct Absolute;

const BACKGROUND_COLOR: Color = Color::Blue;
const RING_MAIN_COLOR: Color = Color::Magenta;
const RING_BACKGROUND_COLOR: Color = Color::Yellow;
const HALF_MAIN_COLOR: Color = Color::White;
const HALF_BACKGROUND_COLOR: Color = Color::Black;

trait CursesExt {
    fn clear_screen(&mut self) -> Option<()>;

    fn text(
        &mut self,
        position: Point2D<i32, Absolute>,
        color_pair: Option<ColorPair>,
        msg: &str,
    ) -> Option<()>;

    fn render_points(
        &mut self,
        display_char: char,
        color_pair: ColorPair,
        points: impl Iterator<Item = Point2D<i32, Absolute>>,
    ) -> Option<()>;

    fn render_valve(&mut self, valve: &Valve) -> Option<()>;

    // At the current cursor location
    fn render_move_list(&mut self, moves: impl Iterator<Item = Move>) -> Option<()>;
}
impl CursesExt for EasyCurses {
    fn clear_screen(&mut self) -> Option<()> {
        // Clear the  screen
        self.clear()?;

        // Paint the background color
        let size = self.get_row_col_count();
        self.set_color_pair(ColorPair::new(BACKGROUND_COLOR, BACKGROUND_COLOR));
        for (r, c) in iproduct!(0..size.0 - 1, 0..size.1) {
            self.move_rc(r, c)?;
            self.print_char(' ')?;
        }
        Some(())
    }

    fn text(
        &mut self,
        position: Point2D<i32, Absolute>,
        color_pair: Option<ColorPair>,
        msg: &str,
    ) -> Option<()> {
        self.set_color_pair(color_pair.unwrap_or(ColorPair::new(Color::White, BACKGROUND_COLOR)));
        self.move_rc(position.y, position.x)?;
        self.print(msg)
    }

    fn render_points(
        &mut self,
        display_char: char,
        color_pair: ColorPair,
        points: impl Iterator<Item = Point2D<i32, Absolute>>,
    ) -> Option<()> {
        for point in points {
            self.move_rc(point.y, point.x)?;
            self.set_color_pair(color_pair);
            self.print_char(display_char)?;
        }

        Some(())
    }

    fn render_valve(&mut self, valve: &Valve) -> Option<()> {
        const START_X: i32 = 1;
        const INSIDE_TRANS: Translation2D<i32, Inside, Absolute> =
            Translation2D::new(START_X + 1, valve::VALVE_MAX_HALF_Y_DISPLACEMENT);
        const OUTSIDE_TRANS: Translation2D<i32, Outside, Absolute> = Translation2D::new(
            START_X + 1 + VALVE_SECTION_WIDTH + 5,
            valve::VALVE_MAX_HALF_Y_DISPLACEMENT,
        );
        const HANAYAMA_HALF_CHAR: char = 'H';
        const VALVE_HALF_CHAR: char = 'V';
        const INNER_RING_CHAR: char = 'I';
        const OUTER_RING_CHAR: char = 'O';

        // Render inside section
        self.render_points(
            HANAYAMA_HALF_CHAR,
            ColorPair::new(HALF_MAIN_COLOR, HALF_BACKGROUND_COLOR),
            valve
                .hanayama_half
                .inside_points()
                .iter()
                .map(|p| INSIDE_TRANS.transform_point(*p)),
        )?;
        self.render_points(
            VALVE_HALF_CHAR,
            ColorPair::new(HALF_BACKGROUND_COLOR, HALF_MAIN_COLOR),
            valve
                .valve_half
                .inside_points(None)
                .iter()
                .map(|p| INSIDE_TRANS.transform_point(*p)),
        )?;
        self.render_points(
            INNER_RING_CHAR,
            ColorPair::new(RING_MAIN_COLOR, RING_BACKGROUND_COLOR),
            valve
                .inner_ring
                .points(None)
                .map(|p| INSIDE_TRANS.transform_point(p)),
        )?;
        self.render_points(
            ' ',
            ColorPair::new(RING_MAIN_COLOR, RING_BACKGROUND_COLOR),
            [-1, VALVE_SECTION_WIDTH].into_iter().flat_map(|x| {
                (0..VALVE_SECTION_HEIGHT).map(move |y| {
                    INSIDE_TRANS
                        .transform_point(InsidePoint::new(x, y + valve.inner_ring.position().y - 1))
                })
            }),
        )?;

        // Render outside section
        self.render_points(
            HANAYAMA_HALF_CHAR,
            ColorPair::new(HALF_MAIN_COLOR, HALF_BACKGROUND_COLOR),
            valve
                .hanayama_half
                .outside_points()
                .iter()
                .map(|p| OUTSIDE_TRANS.transform_point(*p)),
        )?;
        self.render_points(
            VALVE_HALF_CHAR,
            ColorPair::new(HALF_BACKGROUND_COLOR, HALF_MAIN_COLOR),
            valve
                .valve_half
                .outside_points(None)
                .iter()
                .map(|p| OUTSIDE_TRANS.transform_point(*p)),
        )?;
        self.render_points(
            OUTER_RING_CHAR,
            ColorPair::new(RING_BACKGROUND_COLOR, RING_MAIN_COLOR),
            valve
                .outer_ring
                .points(None)
                .map(|p| OUTSIDE_TRANS.transform_point(p)),
        )?;
        self.render_points(
            ' ',
            ColorPair::new(RING_BACKGROUND_COLOR, RING_MAIN_COLOR),
            [-1, VALVE_SECTION_WIDTH].into_iter().flat_map(|x| {
                (0..VALVE_SECTION_HEIGHT).map(move |y| {
                    OUTSIDE_TRANS.transform_point(OutsidePoint::new(
                        x,
                        y + valve.outer_ring.position().y - 1,
                    ))
                })
            }),
        )?;

        Some(())
    }

    fn render_move_list(&mut self, moves: impl Iterator<Item = Move>) -> Option<()> {
        for muv in moves {
            match &muv {
                Move::InnerRingRotate(_) => {
                    self.set_color_pair(ColorPair::new(RING_MAIN_COLOR, RING_BACKGROUND_COLOR));
                }
                Move::OuterRingRotate(_) => {
                    self.set_color_pair(ColorPair::new(RING_BACKGROUND_COLOR, RING_MAIN_COLOR));
                }
                Move::Vertical {
                    direction: _,
                    pieces: _,
                } => {
                    self.set_color_pair(ColorPair::new(HALF_BACKGROUND_COLOR, HALF_MAIN_COLOR));
                }
            }

            self.print(muv.as_str())?;
            self.set_color_pair(ColorPair::new(Color::White, BACKGROUND_COLOR));
            self.print_char(' ')?;
        }

        Some(())
    }
}

#[derive(Error, Debug)]
enum ValveError {
    #[error("curses function failed")]
    Curses,
    #[error("the command was ambiguous")]
    AmbiguousCommand,
}

/// Unforutnately [`easycurses`] functions usually return [`Option<()>`],
/// so we need a way to convert these to a valve result since we cannot
/// simply use early escape syntax.
trait OptionExt<T> {
    fn ok_valve(self) -> Result<T, ValveError>;
}
impl<T> OptionExt<T> for Option<T> {
    fn ok_valve(self) -> Result<T, ValveError> {
        self.ok_or(ValveError::Curses)
    }
}

struct State {
    valve: Valve,
    record: Record<Move>,
    move_inner: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            valve: Valve::default(),
            record: Record::new(),
            move_inner: true,
        }
    }
}

fn main() -> anyhow::Result<()> {
    // The state
    let mut state = State::default();

    // Setup curses
    let mut curses = EasyCurses::initialize_system().ok_valve()?;
    curses
        .set_cursor_visibility(CursorVisibility::Invisible)
        .ok_valve()?;
    curses.set_input_mode(InputMode::Character).ok_valve()?;
    curses.set_echo(false).ok_valve()?;
    curses.set_keypad_enabled(true).ok_valve()?;

    let mut error_message: Option<String> = None;

    const HUD_START_Y: i32 = VALVE_SECTION_HEIGHT + 2 * VALVE_MAX_HALF_Y_DISPLACEMENT;
    const HUD_HELP_START_Y: i32 = HUD_START_Y + 3;

    loop {
        // Clear and render valve
        curses.clear_screen().ok_valve()?;
        curses.render_valve(&state.valve);

        // Render selected ring
        curses
            .text(Point2D::new(0, HUD_START_Y), None, "Current ring: ")
            .ok_valve()?;
        if state.move_inner {
            curses.set_color_pair(ColorPair::new(RING_MAIN_COLOR, RING_BACKGROUND_COLOR));
            curses.print("Inner").ok_valve()?;
        } else {
            curses.set_color_pair(ColorPair::new(RING_BACKGROUND_COLOR, RING_MAIN_COLOR));
            curses.print("Outer").ok_valve()?;
        }

        // Render available move list
        curses
            .text(Point2D::new(0, HUD_START_Y + 1), None, "Available moves: ")
            .ok_valve()?;
        curses
            .render_move_list(Move::iter().filter(|m| state.valve.can_move(m)))
            .ok_valve()?;

        static HELP_MESSAGES: LazyLock<[&str; 7]> = LazyLock::new(|| {
            [
                "Move current ring: <left or right arrows>",
                "Move valve half: <up or down arrows>",
                "Change current ring: <space bar>",
                "Undo: -",
                "Redo: +",
                "Reset: R",
                "Quit: Q",
            ]
        });

        // Render help messages
        for (dy, msg) in HELP_MESSAGES.iter().copied().enumerate() {
            curses
                .text(
                    Point2D::new(0, HUD_HELP_START_Y + i32::try_from(dy).unwrap()),
                    None,
                    msg,
                )
                .ok_valve()?;
        }

        // Render error message (if any)
        if let Some(m) = error_message.take() {
            curses
                .text(
                    Point2D::new(
                        0,
                        HUD_HELP_START_Y + i32::try_from(HELP_MESSAGES.len()).unwrap(),
                    ),
                    Some(ColorPair::new(Color::White, Color::Red)),
                    &m,
                )
                .ok_valve()?;
        }

        // Update screen
        curses.refresh();

        loop {
            match process_input(&mut state, curses.get_input().ok_valve()?) {
                Processed::Quit => return Ok(()),
                Processed::Nothing => {}
                Processed::Refresh(beep) => {
                    if beep {
                        curses.beep();
                    }
                    break;
                }
                Processed::Error(e) => {
                    error_message = Some(format!("ERROR: {e}"));
                    break;
                }
            }
        }
    }
}

enum Processed {
    Quit,
    Nothing,
    Refresh(bool),
    Error(ValveError),
}

fn process_input(state: &mut State, input: Input) -> Processed {
    fn move_vertical(state: &mut State, dir: VerticalDirection) -> Processed {
        let moves = NonEmptyIndexSet::vertical_piece_iter()
            .map(|pieces| Move::Vertical {
                direction: dir,
                pieces,
            })
            .filter(|muv| state.valve.can_move(muv))
            .collect_vec();

        match moves.len() {
            0 => Processed::Refresh(true),
            1 => {
                state
                    .record
                    .edit(&mut state.valve, moves.into_iter().next().unwrap());
                Processed::Refresh(false)
            }
            _ => Processed::Error(ValveError::AmbiguousCommand),
        }
    }

    fn move_ring(state: &mut State, dir: RingDirection) -> Processed {
        let muv = if state.move_inner {
            Move::InnerRingRotate(dir)
        } else {
            Move::OuterRingRotate(dir)
        };

        Processed::Refresh(if state.valve.can_move(&muv) {
            state.record.edit(&mut state.valve, muv);
            false
        } else {
            true
        })
    }

    match input {
        Input::Character(c) => match c {
            ' ' => {
                state.move_inner = !state.move_inner;
                Processed::Refresh(false)
            }
            '-' => Processed::Refresh(state.record.undo(&mut state.valve).is_none()),
            '+' => Processed::Refresh(state.record.redo(&mut state.valve).is_none()),
            'q' => Processed::Quit,
            'r' => {
                *state = State::default();
                Processed::Refresh(false)
            }
            _ => Processed::Nothing,
        },
        Input::KeyLeft => move_ring(state, RingDirection::Left),
        Input::KeyRight => move_ring(state, RingDirection::Right),
        Input::KeyUp => move_vertical(state, VerticalDirection::Up),
        Input::KeyDown => move_vertical(state, VerticalDirection::Down),
        _ => Processed::Nothing,
    }
}
