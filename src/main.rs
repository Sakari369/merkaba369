extern crate piston_window;
extern crate sprite;
extern crate find_folder;

use std::f64::{ consts };
use piston_window::*;
use cgmath::*;

fn calc_poly_vertex(num_points:u32, angle:f64, radius:f64, vertex_index:u32) -> Point2<f64> {
	let idx = num_points - vertex_index;
	let angle_rad = angle.to_radians() + (2.0*consts::PI / num_points as f64) * idx as f64;

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

    let pos = Point2::new(win_size[0]/2.0, win_size[1]/2.0);
    let color = [1.0, 1.0, 1.0, 1.0];
    let num_points = 3;
    let angle = 60.0;
    let radius = 160.0;

    let poly = [
        calc_poly_vertex(num_points, angle, radius, 0),
        calc_poly_vertex(num_points, angle, radius, 1),
        calc_poly_vertex(num_points, angle, radius, 2),
    ];

    let p0 = Point2::new(pos.x + poly[0].x, pos.y + poly[0].y);
    let p1 = Point2::new(pos.x + poly[1].x, pos.y + poly[1].y);
    let p2 = Point2::new(pos.x + poly[1].x, pos.y + poly[1].y);
    let p3 = Point2::new(pos.x + poly[2].x, pos.y + poly[2].y);
    let line_radius = 1.5;

    while let Some(event) = window.next() {
        window.draw_2d(&event, |ctx, gfx, _device| {
            clear([0.05, 0.05, 0.05, 1.0], gfx);
            line(color, line_radius, [p0.x, p0.y, p1.x, p1.y], ctx.transform, gfx);
            line(color, line_radius, [p1.x, p1.y, p2.x, p2.y], ctx.transform, gfx);
            line(color, line_radius, [p2.x, p2.y, p3.x, p3.y], ctx.transform, gfx);
            line(color, line_radius, [p3.x, p3.y, p0.x, p0.y], ctx.transform, gfx);
        });
     }
}