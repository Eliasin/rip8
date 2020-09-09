use super::cpu::CPU;
use crate::mem::register::RegisterFile;
use crate::io::keys::SDL2Keyboard;
use crate::io::screen::Screen;

use sdl2;
use sdl2::event::Event;
use rocket;
use rocket::State;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::json::Json;

use std::time::{ Instant, Duration };
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashSet;

#[get("/registers")]
fn registers(cpu_lock: State<Arc<Mutex<CPU>>>) -> Json<RegisterFile> {
    let cpu = cpu_lock.lock().unwrap();
    Json(cpu.inspect_register_file())
}

#[get("/memory")]
fn memory(cpu_lock: State<Arc<Mutex<CPU>>>) -> Json<Vec<u8>> {
    let cpu = cpu_lock.lock().unwrap();
    Json(cpu.inspect_memory().to_vec())
}

#[post("/add-pc-breakpoint/<pc>")]
fn add_pc_breakpoint(pc: u16, breakpoints_lock: State<Arc<Mutex<HashSet<u16>>>>) {
    let mut breakpoints = breakpoints_lock.lock().unwrap();
    if !breakpoints.contains(&pc) {
        breakpoints.insert(pc);
    }
}

#[post("/delete-pc-breakpoint/<pc>")]
fn delete_pc_breakpoint(pc: u16, breakpoints_lock: State<Arc<Mutex<HashSet<u16>>>>) {
    let mut breakpoints = breakpoints_lock.lock().unwrap();
    if breakpoints.contains(&pc) {
        breakpoints.remove(&pc);
    }
}

#[derive(PartialEq)]
enum IsPaused {
    Paused,
    Running,
}
#[post("/pause")]
fn pause_emulation(paused_lock: State<Arc<Mutex<IsPaused>>>) {
    let mut paused = paused_lock.lock().unwrap();
    *paused = IsPaused::Paused;
}

#[post("/resume")]
fn resume_emulation(paused_lock: State<Arc<Mutex<IsPaused>>>, step_next_lock: State<Arc<Mutex<CanStepNext>>>) {
    let mut paused = paused_lock.lock().unwrap();
    *paused = IsPaused::Running;

    let mut can_step_next = step_next_lock.lock().unwrap();
    *can_step_next = CanStepNext::StayPaused;
}

#[post("/is-paused")]
fn is_paused(paused_lock: State<Arc<Mutex<IsPaused>>>) -> Json<bool> {
    let paused = paused_lock.lock().unwrap();
    match *paused {
        IsPaused::Paused => Json(true),
        IsPaused::Running => Json(false),
    }
}

enum CanStepNext {
    StepNext,
    StayPaused,
}

#[post("/step-next")]
fn step_next(paused_lock: State<Arc<Mutex<IsPaused>>>, step_next_lock: State<Arc<Mutex<CanStepNext>>>) {
    let paused = paused_lock.lock().unwrap();
    match *paused {
        IsPaused::Paused => {
            let mut step_next = step_next_lock.lock().unwrap();
            *step_next = CanStepNext::StepNext;
        },
        IsPaused::Running => {},
    }
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

        let breakpoints_lock = Arc::new(Mutex::new(HashSet::<u16>::new()));
        let rocket_breakpoints_lock = Arc::clone(&breakpoints_lock);

        let paused_lock = Arc::new(Mutex::<IsPaused>::new(IsPaused::Running));
        let rocket_paused_lock = Arc::clone(&paused_lock);

        let can_step_next_lock = Arc::new(Mutex::<CanStepNext>::new(CanStepNext::StayPaused));
        let rocket_can_step_next_lock = Arc::clone(&can_step_next_lock);

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

                let mut paused = paused_lock.lock().unwrap();
                let mut can_step_next = can_step_next_lock.lock().unwrap();

                match *paused {
                    IsPaused::Paused => {

                        match *can_step_next {
                            CanStepNext::StayPaused => continue,
                            CanStepNext::StepNext => *can_step_next = CanStepNext::StayPaused,
                        }

                    },

                    IsPaused::Running => {}
                }

                let mut cpu = cpu_lock.lock().unwrap();
                let pc = cpu.inspect_register_file().PC;

                let breakpoints = breakpoints_lock.lock().unwrap();

                if breakpoints.contains(&pc) && *paused == IsPaused::Running {
                    *paused = IsPaused::Paused;
                    continue;
                }

                let keyboard_state = SDL2Keyboard::new(event_pump.keyboard_state());
                match cpu.execute_cycle(keyboard_state, &mut screen) {
                    Ok(_) => {},
                    Err(error) => {
                        println!("{}", error);
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
                        .manage(rocket_paused_lock)
                        .manage(rocket_breakpoints_lock)
                        .manage(rocket_can_step_next_lock)
                        .mount("/", routes![add_pc_breakpoint, delete_pc_breakpoint, registers, memory, pause_emulation, resume_emulation, is_paused, step_next])
                        .mount("/", StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")))
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
                    println!("{}", error);
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
