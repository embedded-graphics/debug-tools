//! Render a 1px wide antialiased line using error components and a 255 multiplier.
//!
//! Inspiration from <https://computergraphics.stackexchange.com/a/10675>

use embedded_graphics::{
    mock_display::MockDisplay, pixelcolor::Rgb888, prelude::*, primitives::Line,
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use framework::prelude::*;

#[derive(Debug, Clone, Copy)]
struct MajorMinor<T> {
    major: T,
    minor: T,
}

impl<T> MajorMinor<T> {
    fn new(major: T, minor: T) -> Self {
        Self { major, minor }
    }
}

fn thick_line(
    display: &mut impl DrawTarget<Color = Rgb888, Error = std::convert::Infallible>,
    mut line: Line,
    _width: i32,
) -> Result<(), std::convert::Infallible> {
    let skele_color = Rgb888::MAGENTA;

    // let Line { start, end } = line;

    let orig_start_y = line.start.y;

    // line.start.y <<= 8;
    // line.end.y <<= 8;

    let delta = line.delta();

    let dx = delta.x;
    let dy = delta.y;

    let slope = dy as f32 / dx as f32;

    let mut point = line.start;

    let mut error: f32 = 0.0;

    for _i in 0..=dx {
        let c = skele_color;

        // let bright = (1.0 - (error as f32 / e_minor as f32 * 255.0)).abs() as u32;

        // let c = Rgb888::new(
        //     ((bright * skele_color.r() as u32) / 255) as u8,
        //     ((bright * skele_color.g() as u32) / 255) as u8,
        //     ((bright * skele_color.b() as u32) / 255) as u8,
        // );

        Pixel(Point::new(point.x, point.y), c).draw(display)?;

        error += slope;

        if error > 0.5 {
            point.y += 1;
            error -= 1.0;
        }

        point.x += 1;
    }

    // // let Line { start, end } = line;

    // let orig_start_y = line.start.y;

    // // line.start.y <<= 8;
    // // line.end.y <<= 8;

    // let delta = line.delta();

    // let mut error: i32 = 0;
    // let mut point = line.start;

    // // let dx = delta.major.abs();
    // // let dy = delta.minor.abs();
    // let dx = delta.x;
    // let dy = delta.y;

    // let threshold = dx - 2 * dy;
    // let e_minor = -2 * dx;
    // let e_major = 2 * dy;
    // let length = dx + 1;

    // let skele_color = Rgb888::MAGENTA;

    // let mut py = 0;

    // for _i in 0..length {
    //     let c = skele_color;

    //     // let bright = (1.0 - (error as f32 / e_minor as f32 * 255.0)).abs() as u32;

    //     // let c = Rgb888::new(
    //     //     ((bright * skele_color.r() as u32) / 255) as u8,
    //     //     ((bright * skele_color.g() as u32) / 255) as u8,
    //     //     ((bright * skele_color.b() as u32) / 255) as u8,
    //     // );

    //     Pixel(Point::new(point.x, orig_start_y + (py >> 8)), c).draw(display)?;

    //     println!("{}", (error as f32) / threshold as f32);

    //     if error > threshold {
    //         py += 1 << 8;
    //         error += e_minor;
    //     }

    //     error += e_major;
    //     point.x += 1;
    // }

    Ok(())
}

struct LineDebug {
    start: Point,
    end: Point,
    stroke_width: u32,
}

impl App for LineDebug {
    type Color = Rgb888;
    const DISPLAY_SIZE: Size = Size::new(200, 200);
    // const DISPLAY_SIZE: Size = Size::new(64, 64);

    fn new() -> Self {
        let end = Point::new(
            Self::DISPLAY_SIZE.width as i32 / 2,
            Self::DISPLAY_SIZE.height as i32 / 2,
        );
        Self {
            start: end + Point::new(10, 15),
            end,
            // end: start + Point::new(100, 0),
            stroke_width: 10,
        }
    }

    fn parameters(&mut self) -> Vec<Parameter> {
        vec![
            Parameter::new("start", &mut self.start),
            Parameter::new("end", &mut self.end),
            Parameter::new("stroke", &mut self.stroke_width),
        ]
    }

    fn draw(
        &self,
        display: &mut SimulatorDisplay<Self::Color>,
    ) -> Result<(), std::convert::Infallible> {
        let Point { x: _x0, y: _y0 } = self.start;

        // let width = 2 * self.stroke_width as i32 * f32::sqrt((dx * dx + dy * dy) as f32) as i32;
        // let width = (self.stroke_width as i32).pow(2) * (dx * dx + dy * dy);
        let width = self.stroke_width as i32;

        let _mock_display: MockDisplay<Rgb888> = MockDisplay::new();

        thick_line(display, Line::new(self.start, self.end), width)?;

        // let l = Line::new(self.start, self.end);

        // l.into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, 1))
        //     .draw(&mut display.translated(Point::new(40, 40)))?;

        // l.perpendicular()
        //     .into_styled(PrimitiveStyle::with_stroke(Rgb888::RED, 1))
        //     .draw(&mut display.translated(Point::new(40, 40)))?;

        Ok(())
    }
}

fn main() {
    let settings = OutputSettingsBuilder::new().scale(5).build();
    let window = Window::new("Line debugger", &settings);

    LineDebug::run(window);
}
