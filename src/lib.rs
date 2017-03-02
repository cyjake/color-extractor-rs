extern crate image;

use std::path::Path;
use std::default::Default;
use std::collections::HashMap;

use image::FilterType;
use image::GenericImage;

pub struct Color {
  pub r: i32,
  pub g: i32,
  pub b: i32,
  pub hex: i32,
  pub weight: i32
}

fn round (num: u8, delta: u8) -> i32 {
  let delta = delta as f32;
  let result = ((num as f32 - 1.0) / delta + 0.5).floor() * delta;

  if result < 0.0 || result > 255.0 {
    255
  } else {
    result.floor() as i32
  }
}

fn prefer_saturated(c:&Color) -> i32 {
  (c.r - c.g).pow(2) + (c.r - c.b).pow(2) + (c.g - c.b).pow(2) / 65535 * 50
}

fn find_adjacent(c:&Color, gradients:&HashMap<i32, i32>, delta:u8) -> i32 {
  let delta = delta as i32;

  if c.r > delta {
    let hex = c.hex - (delta << 16);
    if let Some(&gradient) = gradients.get(&hex) {
      return gradient;
    }
  }

  if c.g > delta {
    let hex = c.hex - (delta << 8);
    if let Some(&gradient) = gradients.get(&hex) {
      return gradient;
    }
  }

  if c.b > delta {
    let hex = c.hex - delta;
    if let Some(&gradient) = gradients.get(&hex) {
      return gradient;
    }
  }

  if c.r + delta < 255 {
    let hex = c.hex + (delta << 16);
    if let Some(&gradient) = gradients.get(&hex) {
      return gradient;
    }
  }

  if c.g + delta < 255 {
    let hex = c.hex + (delta << 8);
    if let Some(&gradient) = gradients.get(&hex) {
      return gradient;
    }
  }

  if c.b + delta < 255 {
    let hex = c.hex + delta;
    if let Some(&gradient) = gradients.get(&hex) {
      return gradient;
    }
  }

  c.hex
}

#[derive(Debug)]
pub struct Options {
  pub count: usize,
  pub delta: u8,
  pub reduce_gradients: bool,
  pub favor_saturated: bool,
  pub neglect_yellow_skin: bool
}

impl Default for Options {
  fn default() -> Self {
    Options {
      count: 10,
      delta: 16,
      reduce_gradients: true,
      favor_saturated: false,
      neglect_yellow_skin: false
    }
  }
}

pub fn get_color(fpath:&str, colors:&mut Vec<Color>, opts:Options) {
  let max_width = 150;
  let max_height = 150;
  let delta = opts.delta;

  let mut img = image::open(&Path::new(fpath)).unwrap();
  img = img.resize(max_width, max_height, FilterType::Nearest);
  let (width, height) = img.dimensions();
  let pixels = img.raw_pixels();

  for y in 0..height {
    for x in 0..width {
      let i:usize = ((y * width + x) * 3) as usize;
      let mut c = Color {
        r: round(pixels[i], delta),
        g: round(pixels[i + 1], delta),
        b: round(pixels[i + 2], delta),
        hex: 0,
        weight: 1
      };
      c.hex = (c.r << 16) + (c.g << 8) + c.b;
      match colors.binary_search_by(|c2:&Color| c2.hex.cmp(&c.hex)) {
        Ok(i) => colors[i].weight += 1,
        Err(i) => colors.insert(i, c)
      }
    }
  }

  if opts.reduce_gradients {
    colors.sort_by(|a:&Color, b:&Color| b.weight.cmp(&a.weight));

    let mut gradients = HashMap::new();
    let mut ops = vec![];

    for c in colors.iter_mut() {
      let hex;

      if let Some(&gradient) = gradients.get(&c.hex) {
        hex = gradient;
      } else {
        hex = find_adjacent(&c, &gradients, delta);
        gradients.insert(c.hex, hex);
      }

      if hex != c.hex {
        ops.push((hex, c.weight));
        c.weight = 0;
      }
    }

    for (hex, weight) in ops {
      for c in colors.iter_mut() {
        if c.hex == hex {
          c.weight += weight;
        }
      }
    }

    colors.retain(|c:&Color| c.weight > 0);
  }

  colors.sort_by(|a:&Color, b:&Color| b.weight.cmp(&a.weight));
  colors.truncate(opts.count);

  for c in colors.iter_mut() {
    if opts.favor_saturated {
      c.weight = prefer_saturated(&c) * c.weight;
    }

    if opts.neglect_yellow_skin &&
        c.r > c.b && c.r - c.b < 70 && ((c.r + c.b) / 2 - c.g).abs() < 10 {
      c.weight = (c.weight as f32).sqrt().floor() as i32;
    }
  }
}
