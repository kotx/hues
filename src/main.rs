#![forbid(unsafe_code)]
#![deny(clippy::all)]
mod util;

use std::{env, path};

use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{self, EventHandler};
use ggez::graphics::{Canvas, Color, DrawMode, DrawParam, FontData, Mesh, Rect};
use ggez::{Context, ContextBuilder, GameError, GameResult};
use util::{draw_background, draw_centered_text};

fn main() {
	let mut cb = ContextBuilder::new("hues", "Kot")
		.window_setup(WindowSetup::default().title("0x40 Hues"))
		.window_mode(
			WindowMode::default()
				.dimensions(1920., 1080.)
				.resizable(true)
				.resize_on_scale_factor_change(true),
		);

	if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
		let mut path = path::PathBuf::from(manifest_dir);
		path.push("resources");
		println!("Adding path {path:?}");
		cb = cb.add_resource_path(path);
	}

	let (mut ctx, event_loop) = cb.build().expect("failed to create ggez context");
	let state = GlobalState::new(&mut ctx).expect("failed to create global state");
	event::run(ctx, event_loop, state);
}

#[derive(Debug)]
enum Screen {
	Loading { progress: f32 },
	Error { message: String },
}

impl Default for Screen {
	fn default() -> Self {
		Screen::Loading { progress: 0.0 }
	}
}

#[derive(Debug)]
struct GlobalState {
	screen: Screen,
}

impl GlobalState {
	pub fn new(ctx: &mut Context) -> Result<Self, ggez::GameError> {
		ctx.gfx
			.add_font("PetMe 64", FontData::from_path(&ctx.fs, "/PetMe64.ttf")?);

		Ok(Self {
			screen: Screen::default(),
		})
	}
}

impl EventHandler for GlobalState {
	fn update(&mut self, _ctx: &mut Context) -> GameResult {
		if let Screen::Loading { ref mut progress } = self.screen {
			if *progress < 1. {
				*progress = (*progress + 0.02).clamp(0., 1.);
			}
		}

		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult {
		let mut canvas = Canvas::from_frame(ctx, Color::WHITE);

		let (screen_width, screen_height) = {
			let rect = canvas.scissor_rect();
			(rect.w, rect.h)
		};

		match &self.screen {
			Screen::Loading { progress } => {
				draw_background(
					ctx,
					&mut canvas,
					screen_width,
					screen_height,
					Color::from_rgb(0xDD, 0xDD, 0xDD),
				)?;

				canvas.draw(
					&Mesh::new_rectangle(
						ctx,
						DrawMode::fill(),
						Rect::new(0., 0., *progress * screen_width, screen_height),
						Color::WHITE,
					)?,
					DrawParam::default().color(Color::WHITE),
				);

				let progress_text = format!("0x{:x}", (*progress * 0x40 as f32) as u8);
				draw_centered_text(
					&mut canvas,
					&progress_text,
					screen_width,
					screen_height,
					Color::BLACK,
				);
			}
			Screen::Error { message } => {
				draw_background(ctx, &mut canvas, screen_width, screen_height, Color::RED)?;

				draw_centered_text(
					&mut canvas,
					message,
					screen_width,
					screen_height,
					Color::BLACK,
				);
			}
		}

		canvas.finish(ctx)
	}

	fn on_error(&mut self, _ctx: &mut Context, origin: event::ErrorOrigin, err: GameError) -> bool {
		let message = format!(
			"Error in {origin:?}: {}",
			err.to_string().replace("Custom error: ", "")
		);
		self.screen = Screen::Error { message };

		false
	}
}