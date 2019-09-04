extern crate piston;
extern crate piston_window;
extern crate sprite;
extern crate find_folder;
extern crate interpolation;

use std::f64::{ consts };
use piston_window::*;
use piston::input::{UpdateEvent, RenderEvent};
use cgmath::*;
use sprite::*;
use math::Matrix2d;
use std::rc::Rc;
use interpolation::{Ease, EaseFunction};

use ai_behavior::{
    Action,
    State,
    Sequence,
    Success,
    Wait,
    WaitForever,
    While
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

//draw_line_segment(color, line_radius, interpolation, p1, p2, segment_width, origo_trans, gfx);
/*
                    // Animate the lines .. so that based on the current cycle index, 
                    // What we would like to call:

                    // draw_line_part(time, segment_width, Point1, Point2)
                    // And this would draw a line from .
                    // x1, y1 -> x2, y2
                    // In a manner where only part of the line is drawn
                    // And the starting and ending points are advanced based on the interpolation value
                    // So .. with an interpolation value of 0.5, we would draw a segment of the line
                    // at point 
                    //

                    // Line with width of 10
                    // pt1 |::::|::::| pt2

                    // How would we calculate the start point, end point for the segmented line width ?
                    // pt1 |::--|--::| pt2
                    //        ^___^
                    //         sw

                    // Would need to calculate angle between two points, A
                    // Then calculate the normalized center value between pt1, pt2 === nPt
                    // From interpolation value between 0.0 .. 1.0.
                    // Then from this center position of the line, extend the line
                    // in angles +A and -A with segment_width/2.0 amounts.

fn draw_line_segment(color:[f32; 4], width:f64, interpolation:f64, p1:&Point2<f64>, p2:&Point2<f64>,
                     segment_width:f64, translation:Matrix2d, gfx: &mut Graphics) {
            line(color, width, [p1.x, p1.y, p2.x, p2.y], translation, gfx);
}
*/

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

    let mut scene = Scene::new();
    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into()
    };

    let mut textures = Vec::with_capacity(3);

    let mut texture = Rc::new(Texture::from_path(
        &mut texture_context,
        assets.join("3.png"),
        Flip::None,
        &TextureSettings::new()
    ).unwrap());
    textures.push(texture);

    texture = Rc::new(Texture::from_path(
        &mut texture_context,
        assets.join("6.png"),
        Flip::None,
        &TextureSettings::new()
    ).unwrap());
    textures.push(texture);

    texture = Rc::new(Texture::from_path(
        &mut texture_context,
        assets.join("9.png"),
        Flip::None,
        &TextureSettings::new()
    ).unwrap());
    textures.push(texture);

    let origo = Point2::new(win_size[0]/2.0, win_size[1]/2.0);
    let base_color:[f32; 4] = [1.0, 1.0, 1.0, 1.0];
    let trace_color:[f32; 4] = [0.2, 0.6, 1.0, 1.0];
    let num_points = 3;
    let angle = 60.0;
    let radius = 200.0;

    let poly = [
        calc_poly_vertex(num_points, angle, radius, 0),
        calc_poly_vertex(num_points, angle, radius, 1),
        calc_poly_vertex(num_points, angle, radius, 2),
    ];

    let mut ids = Vec::with_capacity(3);

    let number_scale = 0.20;
    let mut s3 = Sprite::from_texture(textures[0].clone());
    s3.set_position(origo.x + poly[1].x - 32.0, origo.y + poly[1].y);
    s3.set_scale(number_scale, number_scale);
    s3.set_opacity(0.0);
    ids.push(scene.add_child(s3));

    let mut s6 = Sprite::from_texture(textures[1].clone());
    s6.set_position(origo.x + poly[2].x, origo.y + poly[2].y - 38.0);
    s6.set_scale(number_scale, number_scale);
    s6.set_opacity(0.0);
    ids.push(scene.add_child(s6));

    let mut s9 = Sprite::from_texture(textures[2].clone());
    s9.set_position(origo.x + poly[0].x + 33.0, origo.y + poly[0].y);
    s9.set_scale(number_scale, number_scale);
    s9.set_opacity(0.0);
    ids.push(scene.add_child(s9));

    // The numbers show up exactly at specific times, right ?
    // 0.22, 0.50
    let number_fade_time = 0.16;
    let number_show_time = 0.26;
    let line_radius = 1.5;

    let mut elapsed_frames = 0;
    let mut number_vis_time = 0.0;
    let number_cycle_time = 600.0;
    //let number_cycle_time = 3200.0;

    let show_seq = Sequence(vec![
        Action(Ease(EaseFunction::QuadraticIn, Box::new(FadeIn(number_fade_time)))),
        Wait(number_show_time),
        Action(Ease(EaseFunction::QuadraticOut, Box::new(FadeOut(number_fade_time)))),
    ]);

    let cycle_points = [
        Point2::new(poly[1].x, poly[1].y),
        Point2::new(poly[2].x, poly[2].y),
        Point2::new(poly[0].x, poly[0].y),
    ];

    let mut p1 = cycle_points[0];
    let mut p2 = cycle_points[1];

    let mut cycle_index = 0;
    let mut active_sprite_id = ids[cycle_index];

    scene.run(active_sprite_id, &show_seq);

    while let Some(event) = window.next() {
        elapsed_frames = elapsed_frames + 1;

        event.update(|args| {
            scene.event(&event);

            let dt_ms = args.dt * 1000.0;
            number_vis_time = number_vis_time + dt_ms;

            if number_vis_time > number_cycle_time {
                if scene.running_for_child(active_sprite_id) < Some(1) {
                    number_vis_time = 0.0;

                    // Cycle to next number.
                    cycle_index = cycle_index + 1;
                    if cycle_index >= ids.len() {
                        cycle_index = 0;
                    }
                    active_sprite_id = ids[cycle_index];

                    scene.run(active_sprite_id, &show_seq);

                    // Update points that are used to draw segmented line along.
                    match cycle_index {
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

/*
        let foo:f64 = 5.5;
        foo.calc(EaseFunction::BackIn);
        */

        window.draw_2d(&event, |ctx, gfx, _device| {
                clear([0.05, 0.05, 0.05, 1.0], gfx);

                scene.draw(ctx.transform, gfx);

                let origo_trans = ctx.transform.trans(origo.x, origo.y);

                line(base_color, line_radius, [poly[0].x, poly[0].y, poly[1].x, poly[1].y], origo_trans, gfx);
                line(base_color, line_radius, [poly[1].x, poly[1].y, poly[2].x, poly[2].y], origo_trans, gfx);
                line(base_color, line_radius, [poly[2].x, poly[2].y, poly[0].x, poly[0].y], origo_trans, gfx);

                {
                    // So first, do we need the angle ?
                    // We need to know how many units we are going to along from the p1 along p2
                    // For this, we need to know the line delta x and y
                    // This can be calculated by using the angle difference between p1 and p2
                    // once we know the delta angle, we can then draw a triangle from p1 -> p2 

                    // So we know the length of the hypotenuse.
                    // In order to find the beginning point sp1, we need to calculate the sides of a triangle
                    // And add to the original p1
                    let angle_rad = radians_between_points(p1, p2);

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

                    //println!("angle_rad = {} angle_deg = {} angle_delta={} angle_delta_deg = {} xdir = {} ydir={}", 
                    //        angle_rad, angle_rad.to_degrees(), angle_delta, angle_delta.to_degrees(), xdir, ydir);

                    // Allright now we have the inner angle
                    // Now solve the a and b
                    // But we would need to know the length of the line ?
                    // How would we figure out that ?
                    // ...

                    let a = (p2.x - p1.x).abs();
                    let b = (p2.y - p1.y).abs();
                    let c = (a*a + b*b).sqrt();

                    //println!("a = {} b = {} c = {}", a, b, c);

                    // So if we figure out the slope of the line, we should be able to calculate the rise and run needed to
                    // advance the line. Then we advance interpolation * c amount towards both positive rise and run.

                    // And 60 X-units. 
                    // The rise is (60 / 100) on each step of the x axis.
                    // But we would have to directly calculate the step at which we are going to start the line.
                    // So .. we multiple the rise by the (segment_width/2.0), and we get the starting point.
                    // Ending point is that + segment_width/2.0;
                    let rise = b / c;
                    let run = a / c;

                    let mut interpolation = (number_vis_time / number_cycle_time);
                    if interpolation > 1.0 {
                        interpolation = 1.0;
                    }

//                  println!("interpolation = {} segment_length = {}", interpolation, segment_length);
                    //println!("d1 = {}", d1);
                    //println!("rise = {} run = {}", rise, run);
                    //println!("shift_x = {} shift_y = {}", shift_x, shift_y);

                    // The distance along hypotenuse 
                    let sp1;
                    let sp2;

                    // The interpolation value goes from 0 .. 1.0
                    // If the segment part is 0.25 % of the interpolation
                    // The line would have to travel distance of -0.25 .. 1.25
                    // So we need another value that is calculated from the interpolation ?
                    // Need to adjust the line range from the interpolated value to the range of 
                    // -0.25 .. 1.25
                    // max, min, etc.
                    //new_value = ( (old_value - old_min) / (old_max - old_min) ) * (new_max - new_min) + new_min
                    let segment_part = 0.50;
                    let in_val = interpolation;
                    let in_min = 0.0;
                    let in_max = 1.0;
                    let out_max = 1.0 + segment_part;
                    let out_min = -segment_part;
                    let line_interpolation = ((in_val - in_min) / (in_max - in_min)) * (out_max - out_min) + out_min;
                    line_interpolation.calc(EaseFunction::CubicIn);
                    //println!("line_interpolation = {}", line_interpolation);

                    // Calculate beginning point
                    {
                        let mut c_distance = c * line_interpolation - (c * 0.80/2.0);
                        if c_distance < 0.0 {
                            c_distance = 0.0;
                        }
                        let shift_x = c_distance * run * xdir;
                        let shift_y = c_distance * rise * ydir;
                        sp1 = Point2::new(p1.x + shift_x, p1.y + shift_y);
                    }

                    {
                        // How much further should the p2 go ?
                        let mut c_distance = (c * line_interpolation) + (c * 0.80/2.0);
                        if c_distance < 0.0 {
                            c_distance = 0.0;
                        } else if c_distance > c {
                            c_distance = c;
                        }
                        let shift_x = c_distance * run * xdir;
                        let shift_y = c_distance * rise * ydir;
                        sp2 = Point2::new(p1.x + shift_x, p1.y + shift_y);
                    }

                    // How to limit the points ?
                    // 

                    line(trace_color, line_radius, [sp1.x, sp1.y, sp2.x, sp2.y], origo_trans, gfx);
                }
            });
        }
     }