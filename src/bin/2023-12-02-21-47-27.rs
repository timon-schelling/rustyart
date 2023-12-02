use delaunator::{next_halfedge, triangulate, Point, EMPTY};
use nannou::color;
use nannou::color::*;
use nannou::ease::*;
use nannou::geom::*;
use nannou::prelude::*;
use nannou::rand::random_f32;
use ordered_float::OrderedFloat;
use std::f32::consts::PI;
use std::iter::*;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    nannou::app(model).update(update).run();
}

#[derive(Copy, Clone, Debug)]
struct Particle {
    position: Vec2,
    radius: f32,
    target: Vec2,
    target_since: SystemTime,
}

#[derive(Copy, Clone, Debug)]
struct Link {
    a: usize,
    b: usize,
    since: SystemTime,
}

#[derive(Clone, Debug)]
struct Model {
    freeze: bool,
    particles: Vec<Particle>,
    links: Vec<Link>,
}

trait RankeableByDistance {
    fn rank_by_distance(&self, others: &Vec<Self>) -> Vec<Self>
    where
        Self: Sized;
}

impl RankeableByDistance for Particle {
    fn rank_by_distance(&self, others: &Vec<Self>) -> Vec<Self>
    where
        Self: Sized,
    {
        let mut ranking = others.clone();
        ranking
            .sort_by_cached_key(|particle| OrderedFloat(particle.position.distance(self.position)));
        ranking
    }
}

const ORIGIN: Vec2 = Vec2::ZERO;
const RADIUS: f32 = 1600.;
const BACKGROUND_COLOR: Rgba = Alpha {
    color: Rgb {
        red: 1.,
        green: 1.,
        blue: 1.,
        standard: std::marker::PhantomData,
    },
    alpha: 0.03,
};
const PARTICLE_RADIUS: f32 = 25.;
const PARTICLE_NUMBER: i32 = 150;
const PARTICLE_SPEED: f32 = 0.5;
const PARTICLE_TARGET_RADIUS: f32 = 220.;
const PARTICLE_TARGET_TIME: f32 = 8.;
const PARTICLE_DISTANCE_MAX: f32 = 400.;
const LINE_WIGHT: f32 = 12.;

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
            .map(|_| Particle {
                position: random_point_in_radius(&ORIGIN, RADIUS),
                radius: PARTICLE_RADIUS,
                target: random_point_in_radius(&ORIGIN, RADIUS),
                target_since: SystemTime::now(),
            })
            .collect::<Vec<Particle>>(),
        links: vec![],
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    if model.freeze {
        return;
    }

    let particles = model.particles.clone();

    for particle in model.particles.iter_mut() {
        if particle.position.distance(particle.target) <= particle.radius
            || particle.target_since.elapsed().unwrap().as_secs_f32()
                > PARTICLE_TARGET_TIME + (PARTICLE_TARGET_TIME * random_f32())
        {
            loop {
                particle.target =
                    random_point_in_radius(&particle.position, PARTICLE_TARGET_RADIUS);
                if particle.target.distance(ORIGIN) <= RADIUS {
                    break;
                };
            }
            particle.target_since = SystemTime::now();
        }

        let neighbours = particle.rank_by_distance(&particles);
        let neighbour = neighbours.get(1).unwrap();
        let neighbour_distance = particle.position.distance(neighbour.position);
        let neighbour_distance_mapped =
            1. - map_range::<f32, f32>(neighbour_distance, 0., 100., 0., 1.).clamp(0., 1.);
        let neighbour_distance_mapped_eased =
            1. - cubic::ease_out(neighbour_distance_mapped, 0., 1., 1.);

        let target_vec =
            (particle.target - particle.position).normalize() * neighbour_distance_mapped_eased;
        let neighbour_vec = (particle.position - neighbour.position).normalize()
            * (1. - neighbour_distance_mapped_eased);

        particle.position += (target_vec + neighbour_vec).normalize() * PARTICLE_SPEED;
    }

    let points = particles
        .iter()
        .map(|particle| Point {
            x: particle.position.x.to_f64().unwrap(),
            y: particle.position.y.to_f64().unwrap(),
        })
        .collect::<Vec<Point>>();

    let triangulation = triangulate(&points);

    let mut links: Vec<Link> = vec![];

    for i in 0..triangulation.triangles.len() {
        if i > triangulation.halfedges[i] || triangulation.halfedges[i] == EMPTY {
            let a = triangulation.triangles[i];
            let b = triangulation.triangles[next_halfedge(i)];
            let link = match model
                .links
                .iter()
                .find(|link| a == link.a && b == link.b || a == link.b && b == link.a)
            {
                Some(link) => link.clone(),
                None => Link {
                    a: a,
                    b: b,
                    since: SystemTime::now(),
                },
            };
            links.push(link);
        }
    }

    model.links = links;
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

    let gradient = Gradient::with_domain(vec![
        (0.0, hsla(41. / 360., 1., 0.5, 1.)),
        (0.65, hsla(0. / 360., 1., 0.5, 1.)),
        (1.0, hsla(234. / 360., 1., 0.5, 1.)),
    ]);

    for link in model.links.iter() {
        let start = model.particles[link.a].position;
        let end = model.particles[link.b].position;

        let distance = start.distance(end);
        if distance > PARTICLE_DISTANCE_MAX {
            continue;
        }
        let distance_mapped =
            map_range::<f32, f32>(distance, PARTICLE_RADIUS*2., PARTICLE_DISTANCE_MAX, 0., 1.).clamp(0., 1.);
        let distance_mapped_eased = 1. - cubic::ease_out(distance_mapped, 0.01, 1., 1.);

        let since = link.since.elapsed().unwrap().as_secs_f32();
        let since_mapped: f32 = map_range::<f32, f32>(since, 0.0, 1.7, 1., 0.).clamp(0., 1.);
        let since_mapped_eased = 1. - cubic::ease_out(since_mapped, 0., 1., 1.);


        let mut color = gradient.get(1.-(distance_mapped*1.5 - 0.2).clamp(0., 1.));
        color.alpha = ((distance_mapped_eased/2.) * (since_mapped_eased*2.)).clamp(0., 0.85);

        draw.line()
            .color(color)
            .weight(LINE_WIGHT)
            .caps_round()
            .points(start, end);
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
