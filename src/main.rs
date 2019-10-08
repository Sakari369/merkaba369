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

enum TriangleDirection {
    Up,
    Down
}

enum DrawMode {
    Segment369,
    Segment457,
    UpDownCycle
}

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

fn draw_line_segment(p1: Point2<f64>, p2: Point2<f64>, interpolation: f64, 
                     color: [f32; 4], line_radius: f64, 
                     translation: math::Matrix2d, gfx: &mut impl Graphics) {
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
        ydir = -1.0;
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

    // Calculate shift in x, y for given interpolation point along a triangle side.
    let unit_shift = interpolation * side_len;
    let shift_x = unit_shift * run * xdir;
    let shift_y = unit_shift * rise * ydir;

    // Draw cycle segment line.
    let x1 = p1.x;
    let y1 = p1.y;
    let x2 = p1.x + shift_x;
    let y2 = p1.y + shift_y;

    line(color, line_radius, [x1, y1, x2, y2], translation, gfx);
}

fn draw_line_triangle(points:&[Point2<f64>; 3], color:[f32; 4], line_radius:f64,
                      translation: math::Matrix2d, gfx: &mut impl Graphics) {
    line(color, line_radius, [points[0].x, points[0].y, points[1].x, points[1].y], translation, gfx);
    line(color, line_radius, [points[1].x, points[1].y, points[2].x, points[2].y], translation, gfx);
    line(color, line_radius, [points[2].x, points[2].y, points[0].x, points[0].y], translation, gfx);
}

fn load_textures(window:&mut PistonWindow, asset_path:&str) -> Vec<Rc<Texture<gfx_device_gl::Resources>>> {
    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into()
    };

    let mut textures = Vec::with_capacity(6);
    let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder(asset_path).unwrap();

    // Load textures for numbers 369.
    let mut texture = Rc::new(Texture::from_path(&mut texture_context, assets.join("3.png"),
                              Flip::None, &TextureSettings::new()).unwrap());
    textures.push(texture);

    texture = Rc::new(Texture::from_path(&mut texture_context, assets.join("6.png"),
                      Flip::None, &TextureSettings::new()).unwrap());
    textures.push(texture);

    texture = Rc::new(Texture::from_path(&mut texture_context, assets.join("9.png"),
                      Flip::None, &TextureSettings::new()).unwrap());
    textures.push(texture);

    // Load textures for numbers 457.
    texture = Rc::new(Texture::from_path(&mut texture_context, assets.join("4.png"),
                      Flip::None, &TextureSettings::new()).unwrap());
    textures.push(texture);
    texture = Rc::new(Texture::from_path(&mut texture_context, assets.join("5.png"),
                      Flip::None, &TextureSettings::new()).unwrap());
    textures.push(texture);
    texture = Rc::new(Texture::from_path(&mut texture_context, assets.join("7.png"),
                      Flip::None, &TextureSettings::new()).unwrap());
    textures.push(texture);

    textures
}

fn main() {
    let opengl = OpenGL::V3_3;

    let win_size = [900.0, 900.0];

    let mut window: PistonWindow = WindowSettings::new("369", win_size)
        .exit_on_esc(true)
        .graphics_api(opengl)
        .vsync(true)
        .build()
        .unwrap();

    let mut scene = Scene::new();

    let textures = load_textures(&mut window, "assets");

    let origo = Point2::new(win_size[0]/2.0, win_size[1]/2.0);
    let num_points = 3;
    let radius = 260.0;

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

    // Create number sprites.
    let mut sprite_ids = Vec::with_capacity(6);
    let number_scale = 0.20;

    // 3.
    let mut number = Sprite::from_texture(textures[0].clone());
    number.set_position(origo.x + poly369[1].x - 32.0, origo.y + poly369[1].y);
    number.set_scale(number_scale, number_scale);
    number.set_opacity(0.0);
    sprite_ids.push(scene.add_child(number));
    // 6.
    number = Sprite::from_texture(textures[1].clone());
    number.set_position(origo.x + poly369[2].x, origo.y + poly369[2].y - 38.0);
    number.set_scale(number_scale, number_scale);
    number.set_opacity(0.0);
    sprite_ids.push(scene.add_child(number));
    // 9.
    number = Sprite::from_texture(textures[2].clone());
    number.set_position(origo.x + poly369[0].x + 33.0, origo.y + poly369[0].y);
    number.set_scale(number_scale, number_scale);
    number.set_opacity(0.0);
    sprite_ids.push(scene.add_child(number));
    // 4.
    number = Sprite::from_texture(textures[3].clone());
    number.set_position(origo.x + poly457[1].x - 33.0, origo.y + poly457[1].y);
    number.set_scale(number_scale, number_scale);
    number.set_opacity(0.0);
    sprite_ids.push(scene.add_child(number));
    // 5.
    number = Sprite::from_texture(textures[4].clone());
    number.set_position(origo.x + poly457[2].x + 29.0, origo.y + poly457[2].y);
    number.set_scale(number_scale, number_scale);
    number.set_opacity(0.0);
    sprite_ids.push(scene.add_child(number));
    // 7.
    number = Sprite::from_texture(textures[5].clone());
    number.set_position(origo.x + poly457[0].x - 2.0, origo.y + poly457[0].y + 44.0);
    number.set_scale(number_scale, number_scale);
    number.set_opacity(0.0);
    sprite_ids.push(scene.add_child(number));

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
    // Clockwise rotation.
    let cycle_points = [
        Point2::new(poly369[1].x, poly369[1].y),
        Point2::new(poly369[2].x, poly369[2].y),
        Point2::new(poly369[0].x, poly369[0].y),

        Point2::new(poly457[1].x, poly457[1].y),
        Point2::new(poly457[2].x, poly457[2].y),
        Point2::new(poly457[0].x, poly457[0].y),
    ];

    let mut triangle_color:[f32; 4] = [1.0, 1.0, 1.0, 1.0];
    let trace_color:[f32; 4] = [0.2, 0.6, 1.0, 1.0];
    let line_radius = 2.0;

    // For cycling numberes in segment draw mode.
    let mut number_vis_time = 0.0;
    let number_cycle_time = 560.0;

    // For cycling the up and down triangles.
    let mut triangle_vis_time = 0.0;
    let triangle_cycle_time = 820.0;

    let mut number_cycle_index;
    let number_cycle_begin;
    let number_cycle_end;

    let draw_mode = DrawMode::UpDownCycle;
    let mut triangle_dir;
    let mut triangle_opacity:f32 = 0.0;

    // Up = 369.
    // Down = 457.
    match draw_mode {
        DrawMode::Segment369 => {
            triangle_dir = TriangleDirection::Up;
        },
        DrawMode::Segment457 => {
            triangle_dir = TriangleDirection::Down;
        }
        DrawMode::UpDownCycle => {
            triangle_dir = TriangleDirection::Up;
        }
    }

    match triangle_dir {
        TriangleDirection::Up => {
            number_cycle_begin = 0;
            number_cycle_end = 2;
        },
        TriangleDirection::Down => {
            number_cycle_begin = 3;
            number_cycle_end = 5;
        }
    };

    number_cycle_index = number_cycle_begin;

    let mut p1 = cycle_points[number_cycle_begin];
    let mut p2 = cycle_points[number_cycle_begin+1];
    let mut active_sprite_id = sprite_ids[number_cycle_index];

    // Init scene.
    match draw_mode {
        DrawMode::Segment369 | DrawMode::Segment457 => {
            scene.run(active_sprite_id, &number_show_seq);
        },
        DrawMode::UpDownCycle => {
        }
    }

    let mut elapsed_frames = 0;

    while let Some(event) = window.next() {
        elapsed_frames = elapsed_frames + 1;

        event.update(|args| {
            scene.event(&event);

            // Run logic based on draw mode.
            match draw_mode {
                DrawMode::Segment369 | DrawMode::Segment457 => {
                    // Has number been shown on screen enough time ?
                    number_vis_time = number_vis_time + (args.dt * 1000.0);
                    if number_vis_time > number_cycle_time {
                        if scene.running_for_child(active_sprite_id) < Some(1) {
                            number_vis_time = 0.0;

                            // Cycle to next number.
                            number_cycle_index = number_cycle_index + 1;
                            if number_cycle_index > number_cycle_end {
                                number_cycle_index = number_cycle_begin;
                            }
                            active_sprite_id = sprite_ids[number_cycle_index];

                            scene.run(active_sprite_id, &number_show_seq);

                            // Update points that are used to draw segmented line along.
                            match number_cycle_index {
                                // 3 -> 6.
                                0 => { 
                                    p1 = cycle_points[0];
                                    p2 = cycle_points[1];
                                },
                                // 6 -> 9.
                                1 => { 
                                    p1 = cycle_points[1];
                                    p2 = cycle_points[2];
                                },
                                // 9 -> 3.
                                2 => { 
                                    p1 = cycle_points[2];
                                    p2 = cycle_points[0];
                                },

                                // 4 -> 5.
                                3 => { 
                                    p1 = cycle_points[3];
                                    p2 = cycle_points[4];
                                },
                                // 5 -> 7.
                                4 => { 
                                    p1 = cycle_points[4];
                                    p2 = cycle_points[5];
                                },
                                // 7 -> 4.
                                5 => { 
                                    p1 = cycle_points[5];
                                    p2 = cycle_points[3];
                                },
                                _ => {
                                }
                            };
                        }
                    }
                },

                DrawMode::UpDownCycle => {
                    triangle_vis_time = triangle_vis_time + (args.dt * 1000.0);
                    if triangle_vis_time > triangle_cycle_time {
                        triangle_vis_time = 0.0;
                        triangle_opacity = 0.0;

                        match triangle_dir {
                            TriangleDirection::Down => {
                                triangle_dir = TriangleDirection::Up;
                            },
                            TriangleDirection::Up => {
                                triangle_dir = TriangleDirection::Down;
                            },
                        }
                    }
                }
            }
        });

        window.draw_2d(&event, |ctx, gfx, _device| {
                clear([0.0, 0.0, 0.0, 1.0], gfx);

                // Draw sprites.
                scene.draw(ctx.transform, gfx);

                let origo_trans = ctx.transform.trans(origo.x, origo.y);
                triangle_color[3] = triangle_opacity;

                match triangle_dir {
                    TriangleDirection::Up => {
                        draw_line_triangle(&poly369, triangle_color, line_radius, origo_trans, gfx);
                    },
                    TriangleDirection::Down => {
                        draw_line_triangle(&poly457, triangle_color, line_radius, origo_trans, gfx);
                    }
                };

                // Current point in along the line we are advancing.
                match draw_mode {
                    DrawMode::Segment369 | DrawMode::Segment457 => {
                        let interpolation = (number_vis_time / number_cycle_time).calc(EaseFunction::ExponentialInOut);
                        draw_line_segment(p1, p2, interpolation, trace_color, line_radius, origo_trans, gfx);
                    },
                    DrawMode::UpDownCycle => {
                        triangle_opacity = ((triangle_vis_time) as f32 / (triangle_cycle_time/3.0) as f32)
                            .calc(EaseFunction::ExponentialIn);
                    }
                }
            });
        }
     }