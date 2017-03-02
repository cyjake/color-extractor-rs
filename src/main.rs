extern crate image;
extern crate color_extractor;

use color_extractor::Options;

fn main() {
  let opts = Options { delta: 17u8, .. Options::default() };
  let mut colors = vec![];

  color_extractor::get_color("tests/img/37.jpg", &mut colors, opts);

  println!("---------");
  for c in colors {
    println!("{:x} {}", c.hex, c.weight);
  }
}

