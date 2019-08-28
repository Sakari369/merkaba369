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

    let mut window: PistonWindow = WindowSettings::new("369", win_size)
        .exit_on_esc(true)
        .graphics_api(opengl)
        .vsync(true)
        .build()
        .unwrap();

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();
    println!("{:?}", assets);
    let mut glyphs = window.load_font(assets.join("iosevka-term-regular.ttf")).unwrap();

    let origo = Point2::new(win_size[0]/2.0, win_size[1]/2.0);
    let color = [1.0, 1.0, 1.0, 1.0];
    let num_points = 3;
    let angle = 60.0;
    let radius = 160.0;

    let poly = [
        calc_poly_vertex(num_points, angle, radius, 0),
        calc_poly_vertex(num_points, angle, radius, 1),
        calc_poly_vertex(num_points, angle, radius, 2),
    ];

    let line_radius = 1.2;
    let font_size = 38.0;
    let txt = text::Text::new_color([1.0, 1.0, 1.0, 1.0], font_size as u32);

    while let Some(event) = window.next() {
        window.draw_2d(&event, |ctx, gfx, device| {
            clear([0.05, 0.05, 0.05, 1.0], gfx);

            let origo_trans = ctx.transform.trans(origo.x, origo.y);
            line(color, line_radius, [poly[0].x, poly[0].y, poly[1].x, poly[1].y], origo_trans, gfx);
            line(color, line_radius, [poly[1].x, poly[1].y, poly[2].x, poly[2].y], origo_trans, gfx);
            line(color, line_radius, [poly[2].x, poly[2].y, poly[0].x, poly[0].y], origo_trans, gfx);

            let trans3 = origo_trans.trans(poly[1].x - 32.0, poly[1].y + 16.0);
            txt.draw("3", &mut glyphs, &ctx.draw_state, trans3, gfx).unwrap();

            let trans6 = origo_trans.trans(poly[2].x - 11.0, poly[2].y - 16.0);
            txt.draw("6", &mut glyphs, &ctx.draw_state, trans6, gfx).unwrap();

            let trans9 = origo_trans.trans(poly[0].x + 9.0, poly[0].y + 16.0);
            txt.draw("9", &mut glyphs, &ctx.draw_state, trans9, gfx).unwrap();

            // Update glyphs before rendering.
            glyphs.factory.encoder.flush(device);
        });
     }
}