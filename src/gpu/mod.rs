use crate::gpu::{
    config::Config,
    render::{RenderCallback, Renderer},
};
use eframe::{
    egui::{self, Key, PointerButton, Vec2, ViewportCommand},
    CreationContext, Frame, NativeOptions,
};
use egui::{CentralPanel, Sense, Ui, Window};
use egui_wgpu::Callback;

mod config;
mod render;

pub const APP_NAME: &str = "raymart";
static SHADER: &str = include_str!("shader.wgsl");

pub struct App {
    show_ui: bool,
    state: AppState,
}

pub struct AppState {
    cfg: Config,
}

impl App {
    pub fn run() -> eframe::Result<()> {
        eframe::run_native(
            APP_NAME,
            NativeOptions::default(),
            Box::new(|cc| Ok(Box::new(App::new(cc)))),
        )
    }

    pub fn new<'a>(cc: &'a CreationContext<'a>) -> Self {
        let cfg = Config::default();
        let render_state = cc.wgpu_render_state.as_ref().unwrap();
        let size = cc.egui_ctx.content_rect().size();

        render_state
            .renderer
            .write()
            .callback_resources
            .insert(Renderer::new(size, &cfg, SHADER, render_state));

        Self {
            show_ui: true,
            state: AppState { cfg },
        }
    }
}

impl AppState {
    fn render_callback(&mut self, size: Vec2) -> RenderCallback {
        let config = self.cfg.as_shader_config(size);

        RenderCallback { config }
    }

    pub fn paint(&mut self, ui: &mut Ui) {
        use PointerButton::{Primary, Secondary};

        let size = ui.available_size();
        let (rect, resp) = ui.allocate_exact_size(size, Sense::click_and_drag());

        let scale = self.cfg.scale(size);
        if resp.dragged_by(Primary) {
            let drag_motion = resp.drag_delta();
            self.cfg.centre[0] -= drag_motion.x * scale;
            self.cfg.centre[1] -= drag_motion.y * scale;
        } else if resp.clicked_by(Secondary) || resp.dragged_by(Secondary) {
            let pointer_pos = resp.interact_pointer_pos().unwrap();
            let ax = self.cfg.axis_x as usize;
            let ay = self.cfg.axis_y as usize;
            self.cfg.initial_value[ax] =
                (pointer_pos.x - size.x / 2.0) * scale + self.cfg.centre[0];
            self.cfg.initial_value[ay] =
                (pointer_pos.y - size.y / 2.0) * scale + self.cfg.centre[1];
        }

        let scroll = ui.input(|i| i.smooth_scroll_delta);
        self.cfg.zoom += self.cfg.zoom * (scroll.y / 300.0).max(-0.9);

        ui.painter().add(Callback::new_paint_callback(
            rect,
            self.render_callback(size),
        ));
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        let ctx = ui.ctx().clone();

        if ctx.input(|i| i.key_pressed(Key::F11)) {
            let current_fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap());
            ctx.send_viewport_cmd(ViewportCommand::Fullscreen(!current_fullscreen));
        }

        if ctx.input(|i| i.key_pressed(Key::F1)) {
            self.show_ui = !self.show_ui;
        }

        CentralPanel::default()
            .frame(egui::Frame::default().inner_margin(0.0))
            .show(ui, |ui| self.state.paint(ui));

        Window::new(APP_NAME)
            .title_bar(true)
            .open(&mut self.show_ui)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Fullscreen: [F11]");
                    ui.label("Toggle UI: [F1]");
                });

                // other UI elements here
                // ui.separator();
            });

        ctx.request_repaint();
    }
}
