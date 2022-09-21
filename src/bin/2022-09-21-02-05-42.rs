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

#[derive(Clone, Debug)]
struct Particle {
    position: Vec2,
    radius: f32,
    teleport: bool,
    draw_position: Vec2,
}

trait RankeableByDrawDistance {
    fn rank_by_draw_distance(&self, others: Vec<Self>) -> Vec<Self>
    where
        Self: Sized;
}

impl RankeableByDrawDistance for Particle {
    fn rank_by_draw_distance(&self, others: Vec<Self>) -> Vec<Self>
    where
        Self: Sized,
    {
        let mut ranking = others;
        ranking
            .sort_by_cached_key(|particle| OrderedFloat(particle.draw_position.distance(self.draw_position)));
        ranking
    }
}

trait RankeableByDistance {
    fn rank_by_distance(&self, others: Vec<Self>) -> Vec<Self>
    where
        Self: Sized;
}

impl RankeableByDistance for Particle {
    fn rank_by_distance(&self, others: Vec<Self>) -> Vec<Self>
    where
        Self: Sized,
    {
        let mut ranking = others;
        ranking
            .sort_by_cached_key(|particle| OrderedFloat(particle.position.distance(self.position)));
        ranking
    }
}

impl RankeableByDistance for Vec2 {
    fn rank_by_distance(&self, others: Vec<Self>) -> Vec<Self>
    where
        Self: Sized,
    {
        let mut ranking = others;
        ranking.sort_by_cached_key(|vec| OrderedFloat(vec.distance(*self)));
        ranking
    }
}

#[derive(Debug)]
struct Model {
    freeze: bool,
    hunters: Vec<Particle>,
    runners: Vec<Particle>,
}

const ORIGIN: Vec2 = Vec2::ZERO;
const RADIUS: f32 = 1500.0;

fn model(app: &App) -> Model {
    app.new_window()
        .fullscreen()
        .view(view)
        .key_released(key_released)
        .build()
        .unwrap();

    Model {
        freeze: false,
        hunters: (0..200)
            .map(|_| Particle {
                position: random_point_in_radius(&ORIGIN, RADIUS),
                radius: 10.0,
                teleport: false,
                draw_position: ORIGIN,
            })
            .collect::<Vec<Particle>>(),
        runners: (0..500)
            .map(|_| Particle {
                position: random_point_in_radius(&ORIGIN, RADIUS),
                radius: 10.0,
                teleport: false,
                draw_position: ORIGIN,
            })
            .collect::<Vec<Particle>>(),
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {

    if model.freeze {
        return;
    }

    let mut particles = model.hunters.clone();
    particles.append(&mut model.runners.clone());

    let hunters = model.hunters.clone();
    let runners = model.runners.clone();

    for hunter in model.hunters.iter_mut() {
        if hunter.teleport {
            hunter.teleport = false;
            loop {
                hunter.position = random_point_in_radius(&ORIGIN, RADIUS);
                let ranking = hunter.rank_by_distance(particles.clone());
                let other = ranking.get(1).unwrap();
                let distance = hunter.position.distance(other.position);
                if !(distance <= other.radius + hunter.radius) {
                    break;
                };
            }
            continue;
        }

        hunter.draw_position = hunter.position;

        let enemy_ranking = hunter.rank_by_distance(runners.clone());
        let enemy = enemy_ranking.first().unwrap();
        let enemy_distance = hunter.position.distance(enemy.position);

        let origin_distance = hunter.position.distance(ORIGIN);

        if enemy_distance < enemy.radius + hunter.radius || origin_distance > RADIUS {
            hunter.teleport = true;
            continue;
        }

        hunter.position -= (hunter.position - enemy.position).normalize() * 1.0;
    }

    for runner in model.runners.iter_mut() {
        if runner.teleport {
            runner.teleport = false;
            loop {
                runner.position = random_point_in_radius(&ORIGIN, RADIUS);
                let ranking = runner.rank_by_distance(particles.clone());
                let other = ranking.get(1).unwrap();
                let distance = runner.position.distance(other.position);
                if !(distance <= other.radius + runner.radius) {
                    break;
                };
            }
            continue;
        }

        runner.draw_position = runner.position;

        let ally_ranking = runner.rank_by_distance(runners.clone());
        let ally = ally_ranking.get(1).unwrap().clone();
        let ally_distance = runner.position.distance(ally.position);

        let enemy_ranking = runner.rank_by_distance(hunters.clone());
        let enemy = enemy_ranking.first().unwrap();
        let enemy_distance = runner.position.distance(enemy.position);

        let origin_distance = runner.position.distance(ORIGIN);

        if enemy_distance < enemy.radius + runner.radius
            || origin_distance > RADIUS
            || ally_distance < runner.radius + ally.radius
        {
            runner.teleport = true;
            continue;
        }

        runner.position += (runner.position - enemy.position).normalize() * 1.5;
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
        .rgba(0.0, 0.0, 0.0, 0.03);

    for hunter in model.hunters.iter() {
        let mut ally_ranking = hunter.rank_by_draw_distance(model.hunters.clone());
        ally_ranking.remove(0);
        for ally in ally_ranking.iter().take_while(|ally| hunter.draw_position.distance(ally.draw_position) < 200.0 ) {
            draw.line()
                .color(RED)
                .weight(3.0)
                .caps_round()
                .points(hunter.draw_position, ally.draw_position);
        }
    }

    for runner in model.runners.iter() {
        let mut ally_ranking = runner.rank_by_draw_distance(model.runners.clone());
        ally_ranking.remove(0);
        for ally in ally_ranking.iter().take_while(|ally| runner.draw_position.distance(ally.draw_position) < 150.0 ) {
            draw.line()
                .color(BLUE)
                .weight(3.0)
                .caps_round()
                .points(runner.draw_position, ally.draw_position);
        }
    }
    
    if true {
        for hunter in model.hunters.iter() {
            draw.ellipse()
                .radius(hunter.radius)
                .xy(hunter.draw_position)
                .color(YELLOW);
        }

        for runner in model.runners.iter() {
            draw.ellipse()
                .radius(runner.radius)
                .xy(runner.draw_position)
                .color(GREEN);
        }
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
            // if model.freeze {
            //     app.set_loop_mode(LoopMode::loop_once());
            // } else {
            //     app.set_loop_mode(LoopMode::RefreshSync);
            // }
        }
        _ => (),
    }
}

fn random_point_in_radius(o: &Vec2, r: f32) -> Vec2 {
    let r = r * random_f32().sqrt();
    let t = random_f32() * 2.0 * PI;
    vec2(o.x + r * t.cos(), o.y + r * t.sin())
}
