extern crate piston_window;
extern crate sprite;
extern crate find_folder;

use std::f32::{self, consts};
use piston_window::*;
use cgmath::*;

fn calc_poly_vertex(num_points:u32, angle:f32, radius:f32, vertex_index:u32) -> Point2<f32> {
	let idx = num_points - vertex_index;
	let angle_rad = angle.to_radians() + (2.0*consts::PI / num_points as f32) * idx as f32;

	let x = angle_rad.sin() * radius;
	let y = angle_rad.cos() * radius;

	return Point2::new(x, y);
}

fn main() {
    let opengl = OpenGL::V3_3;

    let win_size = [1280.0, 720.0];

    let mut window: PistonWindow = WindowSettings::new("Game", win_size)
        .exit_on_esc(true)
        .graphics_api(opengl)
        .vsync(true)
        .build()
        .unwrap();

    let tri_pos = Point2::new(win_size[0]/2.0, win_size[1]/2.0);
    let color = [1.0, 1.0, 1.0, 1.0];
    let num_points = 3;
    let angle = 60.0;
    let radius = 160.0;
    let poly = [
        calc_poly_vertex(num_points, angle, radius, 0),
        calc_poly_vertex(num_points, angle, radius, 1),
        calc_poly_vertex(num_points, angle, radius, 2),
    ];

    while let Some(event) = window.next() {
        window.draw_2d(&event, |ctx, gfx, _device| {
            clear([0.05, 0.05, 0.05, 1.0], gfx);

            polygon(color, &[
                [tri_pos.x + poly[0].x as f64, tri_pos.y + poly[0].y as f64],
                [tri_pos.x + poly[1].x as f64, tri_pos.y + poly[1].y as f64],
                [tri_pos.x + poly[2].x as f64, tri_pos.y + poly[2].y as f64],
            ], ctx.transform, gfx);
        });
     }
}