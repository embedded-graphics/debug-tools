use embedded_graphics::{
    geometry::PointExt,
    mock_display::MockDisplay,
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{
        common::{LineSide, LinearEquation},
        line::StrokeOffset,
        Line, PrimitiveStyle,
    },
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use framework::prelude::*;
use integer_sqrt::IntegerSquareRoot;

#[derive(Debug, Copy, Clone, PartialEq)]
enum LineOffset {
    Left,
    Center,
    Right,
}

impl LineOffset {
    fn widths(self, width: i32) -> (i32, i32) {
        match width {
            width => {
                match self {
                    Self::Left => (width.saturating_sub(1), 0),
                    Self::Center => {
                        let width = width.saturating_sub(1);

                        // Right-side bias for even width lines. Move mod2 to first item in the
                        // tuple to bias to the left instead.
                        (width / 2, width / 2 + (width % 2))
                    }
                    Self::Right => (width.saturating_sub(1), 0),
                }
            }
        }
    }
}

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

fn floor(x: f32) -> f32 {
    x.floor()
}
fn round(x: f32) -> f32 {
    x.round()
}

fn fract(x: f32) -> f32 {
    x.fract()
}

fn recip_fract(x: f32) -> f32 {
    1.0 - x.fract()
}

fn thickline(
    display: &mut impl DrawTarget<Color = Rgb888, Error = std::convert::Infallible>,
    line: Line,
    width: i32,
) -> Result<(), std::convert::Infallible> {
    let Line { start, end } = line;

    let extents = line.extents(width as u32, StrokeOffset::None);

    let (delta, step, pstep) = {
        let delta = end - start;

        let direction = Point::new(
            if delta.x >= 0 { 1 } else { -1 },
            if delta.y >= 0 { 1 } else { -1 },
        );

        let perp_direction = {
            // let perp_delta = Point::new(delta.y, -delta.x);
            let perp_delta = line.perpendicular();
            let perp_delta = perp_delta.end - perp_delta.start;

            Point::new(
                if perp_delta.x >= 0 { 1 } else { -1 },
                if perp_delta.y >= 0 { 1 } else { -1 },
            )
        };

        // Determine major and minor directions.
        if delta.y.abs() >= delta.x.abs() {
            (
                MajorMinor::new(delta.y, delta.x),
                MajorMinor::new(direction.y_axis(), direction.x_axis()),
                MajorMinor::new(perp_direction.y_axis(), perp_direction.x_axis()),
            )
        } else {
            (
                MajorMinor::new(delta.x, delta.y),
                MajorMinor::new(direction.x_axis(), direction.y_axis()),
                MajorMinor::new(perp_direction.x_axis(), perp_direction.y_axis()),
            )
        }
    };

    let mut point = start;

    let dx = delta.major.abs();
    let dy = delta.minor.abs();

    let threshold = dx - 2 * dy;
    let e_minor = -2 * dx;
    let e_major = 2 * dy;
    let length = dx + 1;
    let mut error = 0i32;

    let skele_color = Rgb888::MAGENTA;
    let mut slope = dy as f32 / dx as f32;
    // dbg!(slope);
    // if slope <= 0.0 {
    //     dbg!("shet");
    //     slope = 1.0;
    // }
    let mut e = 0.0f32;
    // println!("===");

    for _i in 0..length {
        // println!("---");

        let bright = ((1.0 - e.fract()) * 255.0) as u32;

        let delta = Point::new(
            e.floor() as i32 * dx.signum(),
            e.floor() as i32 * dy.signum(),
        );
        let delta = delta.component_mul(step.minor);

        let c = Rgb888::new(
            ((bright * skele_color.r() as u32) / 255) as u8,
            ((bright * skele_color.g() as u32) / 255) as u8,
            ((bright * skele_color.b() as u32) / 255) as u8,
        );
        Pixel(point + delta - step.minor, c).draw(display)?;
        Pixel(point + delta, skele_color).draw(display)?;

        if error > threshold {
            // point += step.minor;
            error += e_minor;
            // println!("...");
        }

        e += slope;
        error += e_major;
        point += step.major;
    }

    // for x in 0..length {
    //     {
    //         let bright = ((1.0 - e.fract()) * 255.0) as u32;

    //         let c = Rgb888::new(
    //             ((bright * skele_color.r() as u32) / 255) as u8,
    //             ((bright * skele_color.g() as u32) / 255) as u8,
    //             ((bright * skele_color.b() as u32) / 255) as u8,
    //         );

    //         Pixel(point, c).draw(display)?;
    //     }

    //     {
    //         let bright = ((e.fract()) * 255.0) as u32;

    //         let c = Rgb888::new(
    //             ((bright * skele_color.r() as u32) / 255) as u8,
    //             ((bright * skele_color.g() as u32) / 255) as u8,
    //             ((bright * skele_color.b() as u32) / 255) as u8,
    //         );

    //         Pixel(point - step.minor, c).draw(display)?;
    //     }

    //     e += gradient;
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

        thickline(display, Line::new(self.start, self.end), width)?;

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
