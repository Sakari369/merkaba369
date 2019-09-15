extern crate piston;
extern crate piston_window;
extern crate sprite;
extern crate find_folder;
extern crate interpolation;

use std::f64::{ consts };
use piston_window::*;
use piston::input::{UpdateEvent};
use cgmath::*;
use sprite::*;
use std::rc::Rc;
use interpolation::{Ease, EaseFunction};

use ai_behavior::{
    Action,
    Sequence,
    Wait,
};

fn radians_between_points (p1:Point2<f64>, p2:Point2<f64>) -> f64 {
	let dx = p2.x - p1.x;
	let dy = p2.y - p1.y;
	let mut angle_rad;

	if dx == 0.0 {
		if dy >= 0.0 {
			angle_rad = consts::FRAC_PI_2;
		} else {
			angle_rad = -consts::FRAC_PI_2;
		}
	} else {
		angle_rad = (dy/dx).atan();
		if dx < 0.0 {
			angle_rad = angle_rad + consts::PI;
		}
	}
	if angle_rad < 0.0 {
		angle_rad = angle_rad + consts::PI*2.0;
	}

	angle_rad
}

fn calc_poly_vertex(num_points:u32, angle:f64, radius:f64, vertex_index:u32) -> Point2<f64> {
	let idx = num_points - vertex_index;
	let angle_rad = angle.to_radians() + (2.0*consts::PI / num_points as f64) * idx as f64;

	let x = angle_rad.sin() * radius;
	let y = angle_rad.cos() * radius;

	Point2::new(x, y)
}

fn main() {
    let opengl = OpenGL::V3_3;

    let win_size = [800.0, 800.0];

    let mut window: PistonWindow = WindowSettings::new("369", win_size)
        .exit_on_esc(true)
        .graphics_api(opengl)
        .vsync(true)
        .build()
        .unwrap();

    let mut scene = Scene::new();
    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into()
    };

    // Create textures for numbers 369.
    let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
    let mut textures = Vec::with_capacity(3);
    let mut texture = Rc::new(Texture::from_path(&mut texture_context, assets.join("3.png"),
                              Flip::None, &TextureSettings::new()).unwrap());
    textures.push(texture);

    texture = Rc::new(Texture::from_path(&mut texture_context, assets.join("6.png"),
                      Flip::None, &TextureSettings::new()).unwrap());
    textures.push(texture);

    texture = Rc::new(Texture::from_path(&mut texture_context, assets.join("9.png"),
                      Flip::None, &TextureSettings::new()).unwrap());
    textures.push(texture);

    let origo = Point2::new(win_size[0]/2.0, win_size[1]/2.0);
    let base_color:[f32; 4] = [1.0, 1.0, 1.0, 1.0];
    let trace_color:[f32; 4] = [0.2, 0.6, 1.0, 1.0];
    let num_points = 3;
    let radius = 200.0;

    // Vertex points for 369 triangle.
    let mut angle = 60.0;
    let poly369 = [
        calc_poly_vertex(num_points, angle, radius, 0),
        calc_poly_vertex(num_points, angle, radius, 1),
        calc_poly_vertex(num_points, angle, radius, 2),
    ];

    // Vertex points for 457 triangle.
    angle = 0.0;
    let poly457 = [
        calc_poly_vertex(num_points, angle, radius, 0),
        calc_poly_vertex(num_points, angle, radius, 1),
        calc_poly_vertex(num_points, angle, radius, 2),
    ];

    // Load number textures.
    let mut sprite_ids = Vec::with_capacity(3);
    let number_scale = 0.20;

    let mut s3 = Sprite::from_texture(textures[0].clone());
    s3.set_position(origo.x + poly369[1].x - 32.0, origo.y + poly369[1].y);
    s3.set_scale(number_scale, number_scale);
    s3.set_opacity(0.0);
    sprite_ids.push(scene.add_child(s3));

    let mut s6 = Sprite::from_texture(textures[1].clone());
    s6.set_position(origo.x + poly369[2].x, origo.y + poly369[2].y - 38.0);
    s6.set_scale(number_scale, number_scale);
    s6.set_opacity(0.0);
    sprite_ids.push(scene.add_child(s6));

    let mut s9 = Sprite::from_texture(textures[2].clone());
    s9.set_position(origo.x + poly369[0].x + 33.0, origo.y + poly369[0].y);
    s9.set_scale(number_scale, number_scale);
    s9.set_opacity(0.0);
    sprite_ids.push(scene.add_child(s9));

    // Number show times.
    let number_fade_time = 0.06;
    let number_show_time = 0.26;

    // Number show animation sequence.
    let number_show_seq = Sequence(vec![
        Action(Ease(EaseFunction::QuadraticIn, Box::new(FadeIn(number_fade_time)))),
        Wait(number_show_time),
        Action(Ease(EaseFunction::QuadraticOut, Box::new(FadeOut(number_fade_time)))),
    ]);

    // The points between which cycle lines are being drawn.
    let cycle_points = [
        Point2::new(poly369[1].x, poly369[1].y),
        Point2::new(poly369[2].x, poly369[2].y),
        Point2::new(poly369[0].x, poly369[0].y),
    ];

    let line_radius = 1.5;
    let mut number_vis_time = 0.0;
    let number_cycle_time = 560.0;
    let mut number_cycle_index = 0;

    let mut p1 = cycle_points[0];
    let mut p2 = cycle_points[1];

    let mut active_sprite_id = sprite_ids[number_cycle_index];
    scene.run(active_sprite_id, &number_show_seq);

    let mut elapsed_frames = 0;

    while let Some(event) = window.next() {
        elapsed_frames = elapsed_frames + 1;

        event.update(|args| {
            scene.event(&event);

            // Has number been shown on screen enough time ?
            number_vis_time = number_vis_time + (args.dt * 1000.0);
            if number_vis_time > number_cycle_time {
                if scene.running_for_child(active_sprite_id) < Some(1) {
                    number_vis_time = 0.0;

                    // Cycle to next number.
                    number_cycle_index = number_cycle_index + 1;
                    if number_cycle_index >= sprite_ids.len() {
                        number_cycle_index = 0;
                    }
                    active_sprite_id = sprite_ids[number_cycle_index];

                    scene.run(active_sprite_id, &number_show_seq);

                    // Update points that are used to draw segmented line along.
                    match number_cycle_index {
                        0 => { 
                            p1 = cycle_points[0];
                            p2 = cycle_points[1];
                        },
                        1 => { 
                            p1 = cycle_points[1];
                            p2 = cycle_points[2];
                        },
                        2 => { 
                            p1 = cycle_points[2];
                            p2 = cycle_points[0];
                        },
                        _ => {
                        }
                    };
                }
            }
        });

        window.draw_2d(&event, |ctx, gfx, _device| {
                clear([0.0, 0.0, 0.0, 1.0], gfx);

                // Draw sprites.
                scene.draw(ctx.transform, gfx);

                let origo_trans = ctx.transform.trans(origo.x, origo.y);

                // Draw the base triangle.
                line(base_color, line_radius, [poly369[0].x, poly369[0].y, poly369[1].x, poly369[1].y], origo_trans, gfx);
                line(base_color, line_radius, [poly369[1].x, poly369[1].y, poly369[2].x, poly369[2].y], origo_trans, gfx);
                line(base_color, line_radius, [poly369[2].x, poly369[2].y, poly369[0].x, poly369[0].y], origo_trans, gfx);

                // Calculate angle between polygon points p1 and p2.
                let angle_rad = radians_between_points(p1, p2);

                // From the angle figure out direction of line advancement along x and y -axis.
                let mut xdir = 0.0;
                let mut ydir = 0.0;
                // 0 .. 90.0
                if angle_rad <= consts::FRAC_PI_2 {
                    xdir = 1.0;
                    ydir = 1.0;
                // 90.0 .. 180.0
                } else if (angle_rad >= consts::FRAC_PI_2) && (angle_rad <= consts::PI) {
                    xdir = -1.0;
                    ydir = 1.0;
                // 180.0 .. 270.0
                } else if (angle_rad >= consts::PI) && (angle_rad <= 3.0*consts::PI/2.0) {
                    xdir = -1.0;
                    ydir = 1.0;
                // 180.0 .. 270.0
                } else if (angle_rad >= 3.0*consts::PI/2.0) && (angle_rad <= 2.0*consts::PI) {
                    xdir = 1.0;
                    ydir = -1.0;
                }

                // Calculate length of the triangle side from the point difference
                // with the pythagoran theorem.
                let a = (p2.x - p1.x).abs();
                let b = (p2.y - p1.y).abs();
                let side_len = (a*a + b*b).sqrt();

                // Calculate rise and run ratio for the line angle.
                let rise = b / side_len;
                let run = a / side_len;

                // Current point in along the line we are advancing.
                let interpolation = (number_vis_time / number_cycle_time).calc(EaseFunction::ExponentialInOut);

                // Calculate shift in x, y for given interpolation point along a triangle side.
                let calc_shifts = |interpolation, side_len, run, rise, xdir, ydir| {
                    let unit_shift = interpolation * side_len;
                    let shift_x = unit_shift * run * xdir;
                    let shift_y = unit_shift * rise * ydir;

                    (shift_x, shift_y)
                };

                let (sx, sy) = calc_shifts(interpolation, side_len, run, rise, xdir, ydir);

                // Draw cycle segment line.
                let x1 = p1.x;
                let y1 = p1.y;
                let x2 = p1.x + sx;
                let y2 = p1.y + sy;
                line(trace_color, line_radius, [x1, y1, x2, y2], origo_trans, gfx);
            });
        }
     }