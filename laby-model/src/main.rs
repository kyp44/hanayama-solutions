#![feature(never_type)]

mod controls;
mod state;

use controls::{Action, Controller, Origin};
use macroquad::prelude::*;
use state::{State, Undo};

const MAZE_SIZE_I32: i32 = 480;
const MARGIN_I32: i32 = 20;
const HUD_HEIGHT: i32 = 80;
const FONT_SIZE: f32 = 30.;
const BACKGROUND_COLOR_HEX: u32 = 0x001a33;

const NUB_RADIUS: f32 = 5.;
const NUBS_DISTANCE: f32 = 90.;
const SHOE_REAR_DISTANCE: f32 = 230.;

const MAZE_SIZE: f32 = MAZE_SIZE_I32 as f32;
const MARGIN: f32 = MARGIN_I32 as f32;
const REAR_CENTER: Vec2 = vec2(MARGIN + MAZE_SIZE / 2., MARGIN + MAZE_SIZE / 2.);
const TIP_CENTER: Vec2 = vec2(3. * MARGIN + MAZE_SIZE * 1.5, MARGIN + MAZE_SIZE / 2.);
const HUD_ORIGIN: Vec2 = vec2(0., MAZE_SIZE + 3. * MARGIN);

fn window_conf() -> Conf {
    Conf {
        window_title: "Laby Puzzle".to_owned(),
        window_width: 2 * MAZE_SIZE_I32 + 4 * MARGIN_I32,
        window_height: MAZE_SIZE_I32 + 3 * MARGIN_I32 + HUD_HEIGHT,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() -> Result<(), anyhow::Error> {
    set_pc_assets_folder("assets");

    let rear_side = load_texture("rear-side.png").await?;
    let tip_side = load_texture("tip-side.png").await?;

    assert_eq!(rear_side.size(), vec2(MAZE_SIZE, MAZE_SIZE));
    assert_eq!(tip_side.size(), vec2(MAZE_SIZE, MAZE_SIZE));

    let mut controller = Controller::default();
    let mut undo = Undo::new(State::default());

    loop {
        clear_background(Color::from_hex(BACKGROUND_COLOR_HEX));

        if let Some(action) = controller.check_for_action() {
            match action {
                Action::Quit => break,
                Action::Undo => {
                    undo.undo();
                }
                _ => {
                    undo.new_state(undo.current().apply_action(action));
                }
            }
        }

        // Draw maze images
        draw_texture(&rear_side, MARGIN, MARGIN, WHITE);

        let tip_physical = controller.current_tip_side().is_physical();
        let mut params = DrawTextureParams::default();
        params.flip_y = tip_physical;

        draw_texture_ex(
            &tip_side,
            rear_side.width() + 3.0 * MARGIN,
            MARGIN,
            WHITE,
            params,
        );

        // Draw nubs/shoe
        let state = undo.current();
        let rear_nub = state.rear_nub_position() + REAR_CENTER;
        let tip_nub = if tip_physical {
            let pos = state.tip_nub_position();
            pos.with_y(-pos.y)
        } else {
            state.tip_nub_position()
        } + TIP_CENTER;
        let shoe = state.shoe_position() + REAR_CENTER;

        draw_circle(rear_nub.x, rear_nub.y, NUB_RADIUS, Origin::RearNub.color());
        draw_circle(tip_nub.x, tip_nub.y, NUB_RADIUS, Origin::TipNub.color());
        draw_circle(shoe.x, shoe.y, NUB_RADIUS, WHITE);

        // Draw HUD
        let origin = controller.current_origin();
        let dimensions = draw_text(
            "Quit: Q, Reset: R, Undo: Backspace",
            HUD_ORIGIN.x,
            HUD_ORIGIN.y,
            FONT_SIZE,
            WHITE,
        );
        draw_text(
            &format!("Origin (tab): {}", origin),
            HUD_ORIGIN.x,
            HUD_ORIGIN.y + dimensions.height,
            FONT_SIZE,
            origin.color(),
        );
        draw_text(
            &format!("Mode (~): {}", controller.current_mode()),
            HUD_ORIGIN.x,
            HUD_ORIGIN.y + 2. * dimensions.height,
            FONT_SIZE,
            WHITE,
        );
        draw_text(
            &format!("Tip side (space): {}", controller.current_tip_side()),
            HUD_ORIGIN.x,
            HUD_ORIGIN.y + 3. * dimensions.height,
            FONT_SIZE,
            WHITE,
        );

        next_frame().await
    }

    Ok(())
}
