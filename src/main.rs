extern crate sdl2;

use sdl2::event::Event;
use sdl2::image::{self, InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};
use sdl2::ttf;
use std::f64::consts::PI;
use std::time::{Duration, Instant};
use std::io;
use std::io::prelude::*;

static WIDTH: u32 = 2560;
static HEIGHT: u32 = 1440;

#[derive(Copy, Clone)]
struct Vec2 {
    x: f64,
    y: f64,
}

struct Car<'a> {
    texture: Texture<'a>,
    pos: Vec2,
    vel: f64,
    angle: f64,
    max_vel: f64,
    rotation_vel: f64,
    acceleration: f64,
}

impl<'a> Car<'a> {
    fn new(texture: Texture<'a>, pos: Vec2, max_vel: f64, rotation_vel: f64) -> Self {
        Car {
            texture,
            pos,
            vel: 0.0,
            angle: 0.0,
            max_vel,
            rotation_vel,
            acceleration: 0.1,
        }
    }

    fn rotate(&mut self, left: bool, right: bool) {
        if left {
            self.angle += self.rotation_vel;
        } else if right {
            self.angle -= self.rotation_vel;
        }
    }

    fn move_forward(&mut self) {
        self.vel = (self.vel - self.acceleration).min(self.max_vel);
        self.update_position();
    }

    fn move_backward(&mut self) {
        self.vel = (self.vel + self.acceleration).max(self.max_vel / 2.0);
        self.update_position();
    }

    fn update_position(&mut self) {
        let radians = self.angle * PI / -180.0;
        self.pos.x += self.vel * radians.cos();
        self.pos.y += self.vel * radians.sin();
    }

    // fn draw(&self, canvas: &mut WindowCanvas) {
    //     let query = self.texture.query();
    //     let dst = Rect::new(self.pos.x as i32, self.pos.y as i32, query.width, query.height);
    //     canvas.copy_ex(&self.texture, None, Some(dst), -self.angle, None, false, false).unwrap();
    // }

    fn draw(&self, canvas: &mut WindowCanvas) {
        let query = self.texture.query();
        let original_w = query.width;
        let original_h = query.height;

        // Skalierungsfaktor (z. B. 0.5 = 50 % der Größe)
        let scale = 0.1;
        let scaled_w = (original_w as f64 * scale) as u32;
        let scaled_h = (original_h as f64 * scale) as u32;

        // Damit das Auto zentriert um seine Mitte rotiert, setzen wir den Mittelpunkt
        let dst = Rect::new(
            (self.pos.x as i32) - (scaled_w as i32 / 2),
            (self.pos.y as i32) - (scaled_h as i32 / 2),
            scaled_w,
            scaled_h,
        );

        // Rotation um die Mitte der Textur
        let center = sdl2::rect::Point::new(scaled_w as i32 / 2, scaled_h as i32 / 2);

        canvas.copy_ex(
            &self.texture,
            None,
            Some(dst),
            -self.angle,
            Some(center),
            false,
            false,
        ).unwrap();
    }
}

fn main() -> Result<(), String> {
    // let stdin = io::stdin();
    //     for line in stdin.lock().lines() {
    //         println!("{}", line.unwrap());
    // }

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let ttf_context = ttf::init().map_err(|e| e.to_string())?;

    image::init(InitFlag::PNG | InitFlag::JPG)?;

    let window = video_subsystem
        .window("RustSTK", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    let track_texture = texture_creator.load_texture("imgs/rennstrecke.jpg")?;
    let car_texture = texture_creator.load_texture("imgs/tuxi.png")?;

    let font = ttf_context.load_font("fonts/arial.ttf", 100)?;

    // let mut car = Car::new(car_texture, Vec2 { x: 390.0, y: 433.0 }, 3.0, 4.0);
    let mut car = Car::new(car_texture, Vec2 { x: 920.0, y: 1350.0 }, 3.0, 4.0);

    let mut event_pump = sdl_context.event_pump()?;
    let mut last_update = Instant::now();
    let frame_duration = Duration::from_secs_f64(1.0 / 60.0);

    let lapcount1 = 0;
    let mut won1 = false;
    let win_text1 = "Player 1 has won!!!";
    let mut count_text = 0;

    'running: loop {
        let now = Instant::now();
        let delta_time = now - last_update;
        if delta_time < frame_duration {
            std::thread::sleep(frame_duration - delta_time);
            continue;
        }
        last_update = now;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                _ => {}
            }
        }

        let keys: Vec<Keycode> = event_pump.keyboard_state().pressed_scancodes().filter_map(Keycode::from_scancode).collect();

        let mut moving = false;

        if keys.contains(&Keycode::A) {
            car.rotate(true, false);
        }
        if keys.contains(&Keycode::D) {
            car.rotate(false, true);
        }
        if keys.contains(&Keycode::W) {
            car.move_forward();
            moving = true;
        }
        if keys.contains(&Keycode::S) {
            car.move_backward();
            moving = true;
        }

        if !moving {
            car.vel *= 0.9;
            car.update_position();
        }

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        canvas.copy(&track_texture, None, None)?;
        car.draw(&mut canvas);

        if won1 && (count_text / 15) % 2 == 0 {
            let surface = font.render(win_text1)
                .blended(Color::GREEN)
                .map_err(|e| e.to_string())?;
            let texture = texture_creator.create_texture_from_surface(&surface)
                .map_err(|e| e.to_string())?;
            canvas.copy(&texture, None, Some(Rect::new(215, 260, 370, 100)))?;
        }

        if lapcount1 >= 6 {
            won1 = true;
            if count_text <= 20 {
                count_text += 1;
            } else if count_text > 5 {
                count_text -= 40;
            }
        }

        canvas.present();
    }

    Ok(())
}
