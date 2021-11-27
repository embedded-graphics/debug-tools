//! Render a 1px wide antialiased line using error components and a 255 multiplier.
//!
//! Inspiration from <https://computergraphics.stackexchange.com/a/10675>

use core::convert::TryFrom;
use embedded_graphics::{
    mock_display::MockDisplay, pixelcolor::Rgb888, prelude::*, primitives::Line,
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use framework::prelude::*;

fn thick_line(
    display: &mut impl DrawTarget<Color = Rgb888, Error = std::convert::Infallible>,
    line: Line,
    _width: i32,
) -> Result<(), std::convert::Infallible> {
    let skele_color = Rgb888::MAGENTA;

    let delta = line.delta();

    let dx = delta.x;
    let dy = delta.y;

    let num = dy * 255;
    let denom = dx;
    // Rounding integer division
    let slope = ((num) + (denom) / 2) / (denom);

    let slope = if let Ok(slope) = u8::try_from(slope) {
        slope
    } else {
        // Most likely cause: gradient is incorrect due to improper swapping of major/minor
        // direction. The slope should always be 1.0 or less, or because we multiply by 255 in this
        // case, 255 or less.
        return Ok(());
    };

    let mut point = line.start;

    let mut error: i32 = 0;

    let threshold = dx - 2 * dy;
    // E_diag
    let e_minor = -2 * dx;
    // E_square
    let e_major = 2 * dy;

    // TODO: Calculate initial brightness
    let mut br: u8 = 255;

    for _i in 0..=dx {
        // AA point above
        let bright = br as u32;
        let c = Rgb888::new(
            ((bright * skele_color.r() as u32) / 255) as u8,
            ((bright * skele_color.g() as u32) / 255) as u8,
            ((bright * skele_color.b() as u32) / 255) as u8,
        );
        Pixel(Point::new(point.x, point.y - 1), c).draw(display)?;

        // AA point below
        let bright = (255 - br) as u32;
        let c = Rgb888::new(
            ((bright * skele_color.r() as u32) / 255) as u8,
            ((bright * skele_color.g() as u32) / 255) as u8,
            ((bright * skele_color.b() as u32) / 255) as u8,
        );
        Pixel(Point::new(point.x, point.y + 1), c).draw(display)?;

        // Line skeleton
        // let bright = (e * 255.0) as u32;
        let bright = 255;
        let c = Rgb888::new(
            ((bright * skele_color.r() as u32) / 255) as u8,
            ((bright * skele_color.g() as u32) / 255) as u8,
            ((bright * skele_color.b() as u32) / 255) as u8,
        );
        Pixel(Point::new(point.x, point.y), c).draw(display)?;

        if error > threshold {
            point.y += 1;
            error += e_minor;
            br = 255;
        }

        error += e_major;
        point.x += 1;
        br = br.saturating_sub(slope);
    }

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
