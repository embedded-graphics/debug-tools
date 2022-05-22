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

// // From <https://gist.github.com/rhyolight/2846020>, linked from <https://stackoverflow.com/questions/849211#comment30489239_849211>
// fn dist(line: Line, point: Point) -> f32 {
//     let Line { start, .. } = line;

//     let Point {
//         x: point_x,
//         y: point_y,
//     } = point;

//     let point_x = point_x as f32;
//     let point_y = point_y as f32;

//     let delta = line.delta();

//     let slope = delta.y as f32 / delta.x as f32;
//     let intercept = start.y as f32 - (slope * start.x as f32);

//     f32::abs(slope * point_x - point_y + intercept) / f32::sqrt(slope.powi(2) + 1.0)
// }

fn dist(line: Line, point: Point) -> f32 {
    let Line {
        start: Point { x: x1, y: y1 },
        end: Point { x: x2, y: y2 },
    } = line;
    let Point { x: x3, y: y3 } = point;

    let delta = line.end - line.start;

    let denom = (delta.x.pow(2) + delta.y.pow(2)) as f32;

    let u = ((x3 - x1) * (x2 - x1) + (y3 - y1) * (y2 - y1)) as f32 / denom;

    let x = x1 as f32 + u * (x2 - x1) as f32;
    let y = y1 as f32 + u * (y2 - y1) as f32;

    let dist = f32::sqrt((x - x3 as f32).powi(2) + (y - y3 as f32).powi(2));

    dist
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

    // Direction to travel to hit pixel next to current line
    let perp_direction = {
        // let perp_delta = Point::new(delta.y, -delta.x);
        let perp_delta = line.perpendicular();
        let perp_delta = perp_delta.end - perp_delta.start;

        if perp_delta.y.abs() > perp_delta.x.abs() {
            Point::new(0, if perp_delta.y >= 0 { 1 } else { -1 })
        } else {
            Point::new(if perp_delta.x >= 0 { 1 } else { -1 }, 0)
        }
    };

    let mut point = start;

    let dx = delta.major.abs();
    let dy = delta.minor.abs();

    let threshold = dx - 2 * dy;
    let e_minor = -2 * dx;
    let e_major = 2 * dy;
    let length = dx + 1;
    let aa_base_color = Rgb888::MAGENTA;
    let mut error = 0;

    let slope = dy as f32 / dx as f32;
    let mut bright = 0.5;

    let swap = false;

    println!("===");

    dbg!(line.delta());

    for _i in 0..length {
        let mul = (bright * 255.0) as u32;

        let aa_color = Rgb888::new(
            ((mul * aa_base_color.r() as u32) / 255) as u8,
            ((mul * aa_base_color.g() as u32) / 255) as u8,
            ((mul * aa_base_color.b() as u32) / 255) as u8,
        );

        Pixel(point, Rgb888::MAGENTA).draw(display)?;

        if !swap {
            Pixel(point + perp_direction, aa_color).draw(display)?;
        }

        bright = (bright - slope).max(0.0);

        if error > threshold {
            point += step.minor;
            error += e_minor;
            bright = 1.0;
        }

        error += e_major;
        point += step.major;
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
