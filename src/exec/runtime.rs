use super::cpu::CPU;
use crate::io::keys::SDL2Keyboard;
use crate::io::screen::Screen;

use sdl2;
use sdl2::event::Event;
use rocket;
use rocket::State;

use std::time::{ Instant, Duration };
use std::sync::{Arc, Mutex};
use std::thread;

#[get("/registers")]
fn registers(cpu_lock: State<Arc<Mutex<CPU>>>) -> String {
    let cpu = cpu_lock.lock().unwrap();
    format!("{:?}", cpu.inspect_register_file())
}

#[get("/memory")]
fn memory(cpu_lock: State<Arc<Mutex<CPU>>>) -> String {
    let cpu = cpu_lock.lock().unwrap();
    format!("{:?}", cpu.inspect_memory())
}

pub struct Runtime {}

pub const TIMER_HZ: f64 = 60.0;

fn draw_to_canvas(canvas: &mut sdl2::render::WindowCanvas, screen: &Screen) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for (row, row_arr) in screen.inspect_screen().iter().enumerate() {
        for (column, pixel) in row_arr.iter().enumerate() {
            if *pixel {
                canvas.set_draw_color(sdl2::pixels::Color::BLACK);
                canvas.draw_point(sdl2::rect::Point::new(column as i32, row as i32))?;
            } else {
                canvas.set_draw_color(sdl2::pixels::Color::WHITE);
                canvas.draw_point(sdl2::rect::Point::new(column as i32, row as i32))?;
            }
        }
    }
    Ok(())
}

impl Runtime {
    pub fn new() -> Runtime {
        Runtime{}
    }

    pub fn start_debug(&mut self, program: Vec<u8>, cpu_clock_speed: f64) -> Result<(), Box<dyn::std::error::Error>> {
        let cpu_lock = Arc::new(Mutex::new(CPU::new()));
        let rocket_cpu_lock = Arc::clone(&cpu_lock);

        thread::spawn(move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let sdl_context = sdl2::init()?;
            let mut event_pump = sdl_context.event_pump()?;

            let video_subsystem = sdl_context.video()?;

            let window = video_subsystem.window("rip8", 1280, 640).build()?;
            let mut canvas = window.into_canvas().build()?;

            canvas.set_draw_color(sdl2::pixels::Color::WHITE);
            canvas.clear();
            canvas.set_scale(20.0, 20.0)?;
            canvas.present();

            {
                let mut cpu = cpu_lock.lock().unwrap();
                cpu.map_program(program)?;
                cpu.map_digit_sprites();
            }

            let cpu_time_step: Duration = Duration::new(0, (1000000000.0 / cpu_clock_speed) as u32);

            let mut last_frame_time = Instant::now();
            let mut last_timer_tick = Instant::now();

            let mut screen = Screen::new();

            'running: loop {
                let timer_ticks = ((Instant::now() - last_timer_tick).as_secs_f64() * TIMER_HZ) as u32;
                if timer_ticks > 0 {
                    let mut cpu = cpu_lock.lock().unwrap();
                    last_timer_tick = Instant::now();
                    for _ in 0..timer_ticks {
                        cpu.tick_timers();
                    }
                }

                let next_frame_time = last_frame_time + cpu_time_step;
                if Instant::now() < next_frame_time {
                    continue;
                }

                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit {..} => {
                            break 'running;
                        },
                        _ => {},
                    };
                }

                let mut cpu = cpu_lock.lock().unwrap();
                let keyboard_state = SDL2Keyboard::new(event_pump.keyboard_state());
                match cpu.execute_cycle(keyboard_state, &mut screen) {
                    Ok(_) => {},
                    Err(error) => {
                        println!("{} \n {}", error, cpu.inspect_memory());
                        return Ok(());
                    },
                }

                if screen.has_changed() {
                    draw_to_canvas(&mut canvas, &screen)?;
                    screen.reset_changed();
                    canvas.present();
                }

                last_frame_time = Instant::now();
            };

            Ok(())
        });

        rocket::ignite().manage(rocket_cpu_lock)
                        .mount("/", routes![registers, memory])
                        .launch();
        Ok(())
    }

    pub fn start(&mut self, program: Vec<u8>, cpu_clock_speed: f64) -> Result<(), Box<dyn std::error::Error>> {
        let sdl_context = sdl2::init()?;
        let mut event_pump = sdl_context.event_pump()?;

        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem.window("rip8", 1280, 640).build()?;
        let mut canvas = window.into_canvas().build()?;

        canvas.set_draw_color(sdl2::pixels::Color::WHITE);
        canvas.clear();
        canvas.set_scale(20.0, 20.0)?;
        canvas.present();

        let mut cpu = CPU::new();
        cpu.map_program(program)?;
        cpu.map_digit_sprites();

        let cpu_time_step: Duration = Duration::new(0, (1000000000.0 / cpu_clock_speed) as u32);

        let mut last_frame_time = Instant::now();
        let mut last_timer_tick = Instant::now();

        let mut screen = Screen::new();

        'running: loop {
            let timer_ticks = ((Instant::now() - last_timer_tick).as_secs_f64() * TIMER_HZ) as u32;
            if timer_ticks > 0 {
                last_timer_tick = Instant::now();
                for _ in 0..timer_ticks {
                    cpu.tick_timers();
                }
            }

            let next_frame_time = last_frame_time + cpu_time_step;
            if Instant::now() < next_frame_time {
                continue;
            }

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} => {
                        break 'running;
                    },
                    _ => {},
                };
            }

            let keyboard_state = SDL2Keyboard::new(event_pump.keyboard_state());
            match cpu.execute_cycle(keyboard_state, &mut screen) {
                Ok(_) => {},
                Err(error) => {
                    println!("{} \n {}", error, cpu.inspect_memory());
                    return Ok(());
                },
            }

            if screen.has_changed() {
                match draw_to_canvas(&mut canvas, &screen) {
                    Ok(_) => {},
                    Err(error) => return Err(error),
                };
                screen.reset_changed();
                canvas.present();
            }

            last_frame_time = Instant::now();
        };

        Ok(())
    }
}
