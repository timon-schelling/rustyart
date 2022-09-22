use delaunator::{next_halfedge, triangulate, Point, EMPTY};
use nannou::color::*;
use nannou::ease::*;
use nannou::geom::*;
use nannou::prelude::*;
use nannou::rand::random_f32;
use std::f32::consts::PI;
use std::iter::*;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    nannou::app(model).update(update).run();
}

#[derive(Clone, Debug)]
struct Particle {
    position: Vec2,
    radius: f32,
    target: Vec2,
    draw_position: Vec2,
}

#[derive(Debug)]
struct Model {
    freeze: bool,
    particles: Vec<Particle>,
}

const ORIGIN: Vec2 = Vec2::ZERO;
const RADIUS: f32 = 2000.;
const BACKGROUND_COLOR: Rgba = Alpha {
    color: Rgb{ red: 0., green: 0., blue: 0., standard: std::marker::PhantomData },
    alpha: 0.1,
};
const PARTICLE_RADIUS: f32 = 25.;
const PARTICLE_NUMBER: i32 = 3500;
const PARTICLE_SPEED: f32 = 0.7;
const PARTICLE_TARGET_RADIUS: f32 = 500.;
const PARTICLE_DISTANCE_MAX: f32 = 65.;
const LINE_WIGHT: f32 = 4.5;

fn model(app: &App) -> Model {
    app.new_window()
        .fullscreen()
        .view(view)
        .key_released(key_released)
        .build()
        .unwrap();

    Model {
        freeze: false,
        particles: (0..PARTICLE_NUMBER)
            .map(|_| {
                let position = random_point_in_radius(&ORIGIN, RADIUS);
                let mut target: Vec2;
                loop {
                    target = random_point_in_radius(&position, PARTICLE_TARGET_RADIUS);
                    if target.distance(ORIGIN) <= RADIUS {
                        break;
                    };
                }
                Particle {
                    position: position,
                    radius: PARTICLE_RADIUS,
                    target: target,
                    draw_position: ORIGIN,
                }
            }).collect::<Vec<Particle>>(),
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    if model.freeze {
        return;
    }

    for particle in model.particles.iter_mut() {
        particle.draw_position = particle.position;
        if particle.position.distance(particle.target) <= particle.radius {
            loop {
                particle.target = random_point_in_radius(&particle.position, PARTICLE_TARGET_RADIUS);
                if particle.target.distance(ORIGIN) <= RADIUS {
                    break;
                };
            }
        }
        particle.position += (particle.target - particle.position).normalize() * PARTICLE_SPEED;
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    if model.freeze {
        return;
    }

    let draw = app.draw();
    let win = app.window_rect();
    if app.keys.down.contains(&Key::Delete) {
        draw.background().color(BLACK);
    }

    let win_p = win.pad(0.0);
    draw.rect()
        .xy(win_p.xy())
        .wh(win_p.wh())
        .color(BACKGROUND_COLOR);

    let points = model
        .particles
        .iter()
        .map(|particle| Point {
            x: particle.position.x.to_f64().unwrap(),
            y: particle.position.y.to_f64().unwrap(),
        })
        .collect::<Vec<Point>>();

    let vertecies = model
        .particles
        .iter()
        .map(|particle| particle.position)
        .collect::<Vec<Vec2>>();

    let triangulation = triangulate(&points);

    for i in 0..triangulation.triangles.len() {
        if i > triangulation.halfedges[i] || triangulation.halfedges[i] == EMPTY {
            let start = vertecies[triangulation.triangles[i]];
            let end = vertecies[triangulation.triangles[next_halfedge(i)]];
            let distance = start.distance(end);
            if distance > PARTICLE_DISTANCE_MAX {
                continue;
            }
            let distance_mapped = 1.
                - cubic::ease_out(
                    map_range(distance, 0., PARTICLE_DISTANCE_MAX, 0., 1.),
                    0.,
                    1.,
                    1.,
                );
            let color = hsla(1., 0., 1., distance_mapped);
            draw.line()
                .color(color)
                .weight(LINE_WIGHT)
                .caps_round()
                .points(start, end);
        }
    }

    for particle in model.particles.iter().take(0) {
        draw.ellipse()
            .radius(3.0)
            .xy(particle.draw_position)
            .color(BLACK);
    }

    draw.to_frame(app, &frame).unwrap();
}

fn key_released(app: &App, model: &mut Model, key: Key) {
    match key {
        Key::S => {
            let now = SystemTime::now();
            app.main_window().capture_frame(
                "out/".to_owned()
                    + &app.exe_name().unwrap()
                    + "#"
                    + &now
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                        .to_string()
                    + ".png",
            );
        }
        Key::F => {
            model.freeze = !model.freeze;
        }
        _ => (),
    }
}

fn random_point_in_radius(o: &Vec2, r: f32) -> Vec2 {
    let r = r * random_f32().sqrt();
    let t = random_f32() * 2.0 * PI;
    vec2(o.x + r * t.cos(), o.y + r * t.sin())
}
