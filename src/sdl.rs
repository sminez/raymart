//! SDL2 backed rendering
use anyhow::anyhow;
use sdl2::{
    event::Event,
    pixels::PixelFormatEnum,
    rect::Rect as Sdl2Rect,
    render::{Canvas, Texture, TextureCreator},
    video::{Window, WindowContext},
    EventPump, Sdl, VideoSubsystem,
};

use crate::Color;

pub struct MainThreadState {
    _ctx: Sdl,
    _video_ss: VideoSubsystem,
    evts: EventPump,
}

impl MainThreadState {
    pub fn init(w: u32, h: u32) -> anyhow::Result<(Self, Canvas<Window>)> {
        let ctx = sdl2::init().map_err(|e| anyhow!("{e}"))?;
        let video_ss = ctx.video().map_err(|e| anyhow!("{e}"))?;

        let win = video_ss
            .window("raymart", w, h)
            .position_centered()
            .build()?;

        let canvas = win.into_canvas().target_texture().present_vsync().build()?;
        let evts = ctx.event_pump().map_err(|e| anyhow!("{e}"))?;

        Ok((
            Self {
                _ctx: ctx,
                _video_ss: video_ss,
                evts,
            },
            canvas,
        ))
    }

    /// Poll for currently pending events.
    ///
    /// Returns None if no events are pending.
    pub fn poll_event(&mut self) -> Option<Event> {
        self.evts.poll_event()
    }

    /// Block and wait for the next event.
    ///
    /// Window resize events are handled internally.
    pub fn wait_event(&mut self) -> Event {
        self.evts.wait_event()
    }

    pub fn wait_event_timeout(&mut self, ms: u32) -> Option<Event> {
        self.evts.wait_event_timeout(ms)
    }
}

pub struct Backend<'a> {
    canvas: Canvas<Window>,
    argb_buffer: Vec<u8>,
    screen_texture: Texture<'a>,
    target: Option<Sdl2Rect>,
    logical_w: u32,
}

impl<'a> Backend<'a> {
    pub fn init(
        logical_w: u32,
        logical_h: u32,
        mut canvas: Canvas<Window>,
        tc: &'a TextureCreator<WindowContext>,
    ) -> anyhow::Result<Self> {
        let screen_texture = tc
            .create_texture_streaming(PixelFormatEnum::ARGB8888, logical_w, logical_h)
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        canvas.clear();
        canvas.present();

        Ok(Self {
            canvas,
            argb_buffer: vec![0; logical_w as usize * logical_h as usize * 4],
            screen_texture,
            target: None,
            logical_w,
        })
    }

    pub fn render(&mut self, pixels: &[Color]) -> anyhow::Result<()> {
        for (i, &color) in pixels.iter().enumerate() {
            let base = i * 4;
            let [r, g, b] = color.to_rgb();

            self.argb_buffer[base] = b;
            self.argb_buffer[base + 1] = g;
            self.argb_buffer[base + 2] = r;
            self.argb_buffer[base + 3] = 255;
        }

        self.screen_texture
            .update(None, &self.argb_buffer, self.logical_w as usize * 4)
            .map_err(|e| anyhow!("{e}"))?;

        self.canvas
            .copy(&self.screen_texture, None, self.target)
            .map_err(|e| anyhow!("unable to copy buffer to canvas: {e}"))?;

        self.canvas.present();

        Ok(())
    }
}
