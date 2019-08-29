extern crate piston_window;
extern crate sprite;
extern crate find_folder;

use std::f64::{ consts };
use piston_window::*;
use cgmath::*;
use sprite::*;
use std::rc::Rc;

use ai_behavior::{
    Action,
    State,
    Sequence,
    Success,
    Wait,
    WaitForever,
    While
};

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
    let color = [1.0, 1.0, 1.0, 1.0];
    let num_points = 3;
    let angle = 60.0;
    let radius = 160.0;

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
    let number_fade_time = 0.22;
    let number_show_time = 0.50;

    let show_seq = Sequence(vec![
        Action(Ease(EaseFunction::QuadraticIn, Box::new(FadeIn(number_fade_time)))),
        Wait(number_show_time),
        Action(Ease(EaseFunction::QuadraticOut, Box::new(FadeOut(number_fade_time)))),
    ]);

    let line_radius = 1.5;
    let mut elapsed_frames = 0;
    let mut number_vis_frames = 0;
    let mut id_idx = 0;
    let mut active_sprite_id = ids[id_idx];
    let show_number_frames = 480;
    while let Some(event) = window.next() {
        elapsed_frames = elapsed_frames + 1;
        number_vis_frames = number_vis_frames + 1;

        if number_vis_frames > show_number_frames {
            if scene.running_for_child(active_sprite_id) < Some(1) {
                scene.run(active_sprite_id, &show_seq);
                number_vis_frames = 0;

                id_idx = id_idx + 1;
                if id_idx >= ids.len() {
                    id_idx = 0;
                }
                active_sprite_id = ids[id_idx];
            }
        }

        scene.event(&event);

        window.draw_2d(&event, |ctx, gfx, device| {
            clear([0.05, 0.05, 0.05, 1.0], gfx);

            scene.draw(ctx.transform, gfx);

            let origo_trans = ctx.transform.trans(origo.x, origo.y);
            line(color, line_radius, [poly[0].x, poly[0].y, poly[1].x, poly[1].y], origo_trans, gfx);
            line(color, line_radius, [poly[1].x, poly[1].y, poly[2].x, poly[2].y], origo_trans, gfx);
            line(color, line_radius, [poly[2].x, poly[2].y, poly[0].x, poly[0].y], origo_trans, gfx);
        });
     }
}