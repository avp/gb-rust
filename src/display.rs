extern crate glium;
use self::glium::glutin;
use self::glium::Surface;

use gpu;

const WIDTH: u32 = gpu::WIDTH as u32;
const HEIGHT: u32 = gpu::HEIGHT as u32;
const WINDOW_SCALE: u32 = 4;

pub struct Display {
  pub events_loop: glutin::EventsLoop,

  display: Box<glium::Display>,
  dest_texture: Box<glium::Texture2d>,
}

impl Display {
  pub fn new() -> Display {
    let events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
      .with_dimensions(WIDTH * WINDOW_SCALE, HEIGHT * WINDOW_SCALE)
      .with_title("GB Rust");
    let context = glutin::ContextBuilder::new();

    let display = glium::Display::new(window, context, &events_loop).unwrap();

    let (w, h) = display.get_framebuffer_dimensions();
    let dest_texture = glium::Texture2d::empty_with_format(
      &display,
      glium::texture::UncompressedFloatFormat::U8U8U8U8,
      glium::texture::MipmapsOption::NoMipmap,
      w,
      h,
    ).unwrap();
    dest_texture
      .as_surface()
      .clear_color(0.0, 0.0, 0.0, 1.0);

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 1.0);
    target.finish().unwrap();

    Display {
      events_loop: events_loop,

      display: Box::new(display),
      dest_texture: Box::new(dest_texture),
    }
  }

  pub fn redraw(&self, frame: &gpu::Frame) {
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(
      frame,
      (WIDTH, HEIGHT),
    );

    let texture = glium::Texture2d::new(&*self.display, image).unwrap();

    static DEST_RECT: glium::BlitTarget = glium::BlitTarget {
      left: 0,
      bottom: 0,
      width: (WIDTH * WINDOW_SCALE) as i32,
      height: (HEIGHT * WINDOW_SCALE) as i32,
    };

    texture.as_surface().blit_whole_color_to(
      &self.dest_texture.as_surface(),
      &DEST_RECT,
      glium::uniforms::MagnifySamplerFilter::Linear,
    );

    let target = self.display.draw();
    self.dest_texture.as_surface().fill(
      &target,
      glium::uniforms::MagnifySamplerFilter::Linear,
    );

    target.finish().unwrap();
  }
}
