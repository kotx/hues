#![feature(extract_if)]
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![allow(unused)] // TODO: remove when done prototyping

mod respack;
mod util;

use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::mpsc::{channel, Receiver};
use std::thread::JoinHandle;
use std::time::Duration;
use std::{env, path, thread};

use crate::respack::Respack;
use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{self, EventHandler};
use ggez::graphics::{Canvas, Color, DrawMode, DrawParam, FontData, Mesh, Rect};
use ggez::{Context, ContextBuilder, GameError, GameResult};
use respack::{RespackError, RespackResult};
use util::{draw_background, draw_centered_text};

fn main() {
	let mut cb = ContextBuilder::new("0x40-hues", "Kot")
		.window_setup(WindowSetup::default().title("0x40 Hues of Rust"))
		.window_mode(
			WindowMode::default()
				.dimensions(1280., 720.)
				.resizable(true)
				.resize_on_scale_factor_change(true),
		);

	let (mut ctx, event_loop) = cb.build().expect("failed to create ggez context");
	let mut state = GlobalState::new(&mut ctx).expect("failed to create global state");

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
struct RespackLoader {
	thread: JoinHandle<RespackResult<Respack>>,
	path: path::PathBuf,
	pub progress: Receiver<f32>,
}

impl RespackLoader {
	pub fn new(path: impl AsRef<Path>) -> Self {
		let (sender, recv) = channel();
		let path = path.as_ref().to_path_buf();
		let path_clone = path.clone();

		let thread = thread::spawn(move || -> RespackResult<Respack> {
			let mut progress = 0.0;

			let respack = Respack::load_from_file(path_clone)?;

			// loop {
			// 	progress += 0.04;
			// 	sender.send(progress).unwrap();
			// 	thread::sleep(Duration::from_millis(10));

			// 	if progress >= 1.0 {
			// 		break;
			// 	}
			// }

			Ok(respack)
		});

		RespackLoader {
			thread,
			path,
			progress: recv,
		}
	}
}

#[derive(Debug)]
struct GlobalState {
	screen: Screen,
	loading_respacks: Vec<RespackLoader>,
	respacks: Vec<Respack>,
}

impl GlobalState {
	pub fn new(ctx: &mut Context) -> Result<Self, GameError> {
		ctx.gfx
			.add_font("Pet Me 64", FontData::from_path(&ctx.fs, "/PetMe64.ttf")?);

		let loading_respacks = ctx
			.fs
			.read_dir("/respacks")?
			.filter(|p| p.is_file())
			.map(RespackLoader::new)
			.collect();

		Ok(Self {
			screen: Screen::default(),
			loading_respacks,
			respacks: Vec::new(),
		})
	}
}

impl EventHandler for GlobalState {
	fn update(&mut self, _ctx: &mut Context) -> GameResult {
		if let Screen::Loading { ref mut progress } = self.screen {
			let finished = self
				.loading_respacks
				.extract_if(|loader| loader.thread.is_finished());

			for loader in finished {
				let join_result = loader.thread.join().unwrap();
				let loader_path = loader.path;

				match join_result {
					Ok(respack) => {
						println!("Loaded respack: {respack:?}");
						self.respacks.push(respack);
					}
					Err(err) => {
						println!("Error loading respack {loader_path:?}: {err:?}");
						return Err(GameError::CustomError(format!("{err:?}")));
					}
					_ => (),
				}
			}

			for loader in &self.loading_respacks {
				let new_progress = loader.progress.try_recv();
				*progress += new_progress.unwrap_or(0.0);
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
