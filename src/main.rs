#[macro_use]
extern crate strum_macros;

use std::{env, mem};
use std::path;
use std::time::Instant;

use log::*;
use pretty_env_logger::env_logger::Env;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use crate::common::Direction;
use crate::context::Context;
use crate::error::GameResult;
use crate::game::caret::{Caret, CaretType};
use crate::game::engine_constants::EngineConstants;
use crate::game::stage::StageData;
use crate::rng::RNG;
use crate::scene::loading_scene::LoadingScene;
use crate::scene::Scene;
use crate::sound::SoundManager;
use crate::texture_set::TextureSet;
use crate::ui::UI;

mod common;
mod context;
mod error;
mod game;
mod live_debugger;
mod renderer;
mod rng;
mod scene;
mod sound;
mod texture_set;
mod ui;

bitfield! {
  pub struct KeyState(u16);
  impl Debug;
  left, set_left: 0;
  right, set_right: 1;
  up, set_up: 2;
  down, set_down: 3;
  map, set_map: 4;
  jump, set_jump: 5;
  fire, set_fire: 6;
  weapon_next, set_weapon_next: 7;
  weapon_prev, set_weapon_prev: 8;
}

bitfield! {
  pub struct GameFlags(u32);
  impl Debug;
  pub flag_x01, set_flag_x01: 0;
  pub control_enabled, set_control_enabled: 1;
  pub flag_x04, set_flag_x04: 2;
}

struct Game {
    scene: Option<Box<dyn Scene>>,
    state: SharedGameState,
    ui: UI,
}

pub struct SharedGameState {
    pub flags: GameFlags,
    pub game_rng: RNG,
    pub effect_rng: RNG,
    pub carets: Vec<Caret>,
    pub key_state: KeyState,
    pub key_trigger: KeyState,
    pub texture_set: TextureSet,
    pub base_path: String,
    pub stages: Vec<StageData>,
    pub sound_manager: SoundManager,
    pub constants: EngineConstants,
    pub scale: f32,
    pub canvas_size: (f32, f32),
    pub screen_size: (f32, f32),
    pub next_scene: Option<Box<dyn Scene>>,
    key_old: u16,
}

impl SharedGameState {
    pub fn update_key_trigger(&mut self) {
        let mut trigger = self.key_state.0 ^ self.key_old;
        trigger = self.key_state.0 & trigger;
        self.key_old = self.key_state.0;
        self.key_trigger = KeyState(trigger);
    }

    pub fn tick_carets(&mut self) {
        for caret in self.carets.iter_mut() {
            caret.tick(&self.effect_rng, &self.constants);
        }

        self.carets.retain(|c| !c.is_dead());
    }

    pub fn create_caret(&mut self, x: isize, y: isize, ctype: CaretType, direct: Direction) {
        self.carets.push(Caret::new(x, y, ctype, direct, &self.constants));
    }
}

impl Game {
    fn new(ctx: &mut Context) -> GameResult<Game> {
        let scale = 2.0;
        let screen_size = graphics::drawable_size(ctx);
        let canvas_size = (screen_size.0 / scale, screen_size.1 / scale);
        let mut constants = EngineConstants::defaults();
        let mut base_path = "/";

        if filesystem::exists(ctx, "/base/Nicalis.bmp") {
            info!("Cave Story+ data files detected.");
            constants.apply_csplus_patches();
            base_path = "/base/";
        } else if filesystem::exists(ctx, "/mrmap.bin") || filesystem::exists(ctx, "/Font/font") {
            info!("CSE2E data files detected.");
        } else if filesystem::exists(ctx, "/stage.dat") || filesystem::exists(ctx, "/sprites.sif") {
            info!("NXEngine-evo data files detected.");
        }

        let s = Game {
            scene: None,
            ui: UI::new(ctx)?,
            state: SharedGameState {
                flags: GameFlags(0),
                game_rng: RNG::new(0),
                effect_rng: RNG::new(Instant::now().elapsed().as_nanos() as i32),
                carets: Vec::with_capacity(32),
                key_state: KeyState(0),
                key_trigger: KeyState(0),
                texture_set: TextureSet::new(base_path),
                base_path: str!(base_path),
                stages: Vec::with_capacity(96),
                sound_manager: SoundManager::new(ctx),
                constants,
                scale,
                screen_size,
                canvas_size,
                next_scene: None,
                key_old: 0,
            },
        };

        Ok(s)
    }

    fn tick(&mut self, ctx: &mut Context) -> GameResult {
        if let Some(scene) = self.scene.as_mut() {
            scene.tick(&mut self.state, ctx)?;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());
        graphics::set_transform(ctx, self.scaled_matrix);
        graphics::apply_transformations(ctx)?;

        if let Some(scene) = self.scene.as_mut() {
            scene.draw(&mut self.state, ctx)?;

            graphics::set_transform(ctx, self.def_matrix);
            graphics::apply_transformations(ctx)?;
            self.ui.draw(&mut self.state, ctx, scene)?;
        }

        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, key_code: VirtualKeyCode, repeat: bool) {
        if repeat { return; }

        // todo: proper keymaps?
        let state = &mut self.state;
        match key_code {
            VirtualKeyCode::Left => { state.key_state.set_left(true) }
            VirtualKeyCode::Right => { state.key_state.set_right(true) }
            VirtualKeyCode::Up => { state.key_state.set_up(true) }
            VirtualKeyCode::Down => { state.key_state.set_down(true) }
            VirtualKeyCode::Z => { state.key_state.set_jump(true) }
            VirtualKeyCode::X => { state.key_state.set_fire(true) }
            VirtualKeyCode::A => { state.key_state.set_weapon_prev(true) }
            VirtualKeyCode::S => { state.key_state.set_weapon_next(true) }
            _ => {}
        }
    }


    fn key_up_event(&mut self, _ctx: &mut Context, key_code: VirtualKeyCode) {
        let state = &mut self.state;

        match key_code {
            VirtualKeyCode::Left => { state.key_state.set_left(false) }
            VirtualKeyCode::Right => { state.key_state.set_right(false) }
            VirtualKeyCode::Up => { state.key_state.set_up(false) }
            VirtualKeyCode::Down => { state.key_state.set_down(false) }
            VirtualKeyCode::Z => { state.key_state.set_jump(false) }
            VirtualKeyCode::X => { state.key_state.set_fire(false) }
            VirtualKeyCode::A => { state.key_state.set_weapon_prev(false) }
            VirtualKeyCode::S => { state.key_state.set_weapon_next(false) }
            _ => {}
        }
    }
}

pub fn main() -> GameResult {
    pretty_env_logger::env_logger::init_from_env(Env::default().default_filter_or("info"));

    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("data");
        path
    } else {
        path::PathBuf::from(&env::var("CAVESTORY_DATA_DIR").unwrap_or(str!("data")))
    };

    info!("Resource directory: {:?}", resource_dir);
    info!("Initializing engine...");

    let event_loop = EventLoop::new();
    let ctx = &mut Context::new();
    let game = &mut Game::new(ctx)?;
    game.state.next_scene = Some(Box::new(LoadingScene::new()));

    event_loop.run(move |event, _, control_flow| {
        game.ui.handle_events(ctx, &event);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input:
                    KeyboardInput {
                        state: el_state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                    ..
                } => {
                    match el_state {
                        ElementState::Pressed => {
                            ctx.keyboard_mut().set_key_state(keycode, true);
                            game.key_down_event(ctx, keycode, ctx.keyboard().is_key_repeated());
                        }
                        ElementState::Released => {
                            ctx.keyboard_mut().set_key_state(keycode, false);
                            game.key_up_event(ctx, keycode);
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }


        game.tick(ctx)?;
        game.draw(ctx)?;

        if game.state.next_scene.is_some() {
            mem::swap(&mut game.scene, &mut game.state.next_scene);
            game.state.next_scene = None;

            game.scene.as_mut().unwrap().init(&mut game.state, ctx)?;
        }
    });
}
