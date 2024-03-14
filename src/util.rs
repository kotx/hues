use ggez::{
	glam::Vec2,
	graphics::{Canvas, Color, DrawMode, DrawParam, Mesh, Rect, Text, TextLayout},
	Context, GameResult,
};

pub fn draw_background(
	ctx: &mut Context,
	canvas: &mut Canvas,
	screen_width: f32,
	screen_height: f32,
	color: Color,
) -> GameResult {
	canvas.draw(
		&Mesh::new_rectangle(
			ctx,
			DrawMode::fill(),
			Rect::new(0., 0., screen_width, screen_height),
			color,
		)?,
		DrawParam::default(),
	);

	Ok(())
}

pub fn draw_centered_text(
	canvas: &mut Canvas,
	text: &str,
	screen_width: f32,
	screen_height: f32,
	color: Color,
) {
	canvas.draw(
		Text::new(text)
			.set_font("Pet Me 64")
			.set_scale(48.)
			.set_layout(TextLayout::center())
			.set_bounds(Vec2::new(screen_width, screen_height))
			.set_wrap(true),
		DrawParam::default()
			.color(color)
			.dest(Vec2::new(screen_width / 2., screen_height / 2.)),
	);
}
