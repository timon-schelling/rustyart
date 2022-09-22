use delaunator::{triangulate, next_halfedge, Point, EMPTY};
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
const RADIUS: f32 = 1700.0;
const TARGET_RADIUS: f32 = 150.0;

fn model(app: &App) -> Model {
    app.new_window()
        .fullscreen()
        .view(view)
        .key_released(key_released)
        .build()
        .unwrap();

    Model {
        freeze: false,
        particles: (0..400)
            .map(|_| Particle {
                position: random_point_in_radius(&ORIGIN, RADIUS),
                radius: 25.0,
                target: random_point_in_radius(&ORIGIN, RADIUS),
                draw_position: ORIGIN,
            })
            .collect::<Vec<Particle>>(),
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
                particle.target = random_point_in_radius(&particle.position, TARGET_RADIUS);
                if particle.target.distance(ORIGIN) <= RADIUS {
                    break;
                };
            }
        }
        particle.position -= (particle.position - particle.target).normalize() * 0.2;
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
        .rgba(0.0, 0.0, 0.0, 0.04);

    let points = model
        .particles
        .iter()
        .map(|particle| Point {
            x: particle.position.x.to_f64().unwrap(),
            y: particle.position.y.to_f64().unwrap(),
        })
        .collect::<Vec<Point>>();

    let vertecies = model.particles.iter().map(|particle| particle.position).collect::<Vec<Vec2>>();
    
    let triangulation = triangulate(&points);

    for i in 0..triangulation.triangles.len() {
        if i > triangulation.halfedges[i] || triangulation.halfedges[i] == EMPTY {
            let start = vertecies[triangulation.triangles[i]];
            let end = vertecies[triangulation.triangles[next_halfedge(i)]];
            let distance = start.distance(end);
            let distance_mapped = map_range(distance, 50.0, 500.0, 0.0, 1.0);
            let color = hsva( 1.0 - distance_mapped, 1.0, 1.0, 1.0 - distance_mapped);
            draw.line()
                .color(color)
                .weight(4.5)
                .caps_round()
                .points(start, end);
        }
    }

    for particle in model.particles.iter() {
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
