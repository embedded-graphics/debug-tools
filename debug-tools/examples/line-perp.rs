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

// From <https://gist.github.com/rhyolight/2846020>, linked from <https://stackoverflow.com/questions/849211#comment30489239_849211>
fn dist(line: Line, point: Point) -> f32 {
    let Line { start, .. } = line;

    let Point {
        x: point_x,
        y: point_y,
    } = point;

    let point_x = point_x as f32;
    let point_y = point_y as f32;

    let delta = line.delta();

    let slope = delta.y as f32 / delta.x as f32;
    let intercept = start.y as f32 - (slope * start.x as f32);

    f32::abs(slope * point_x - point_y + intercept) / f32::sqrt(slope.powi(2) + 1.0)
}

fn perpendicular(
    display: &mut impl DrawTarget<Color = Rgb888, Error = std::convert::Infallible>,
    line: Line,
    (left_extent, right_extent): (Line, Line),
    x0: i32,
    y0: i32,
    delta: MajorMinor<i32>,
    step: MajorMinor<Point>,
    einit: i32,
    width: i32,
    winit: i32,
    extra: bool,
) -> Result<(), std::convert::Infallible> {
    let mut point = Point::new(x0, y0);

    if width == 0 {
        return Ok(());
    }

    let dx = delta.major;
    let dy = delta.minor;

    let sign = match (step.major, step.minor) {
        (Point { x: -1, y: 0 }, Point { x: 0, y: 1 }) => -1,
        (Point { x: 0, y: -1 }, Point { x: -1, y: 0 }) => -1,
        (Point { x: 1, y: 0 }, Point { x: 0, y: -1 }) => -1,
        (Point { x: 0, y: 1 }, Point { x: 1, y: 0 }) => -1,
        _ => 1,
    };

    let dx = dx.abs();
    let dy = dy.abs();

    let threshold = dx - 2 * dy;
    // E_diag
    let e_minor = -2 * dx;
    // E_square
    let e_major = 2 * dy;

    let mut error = einit * sign;

    let (side_check_left, side_check_right) = (LineSide::Left, LineSide::Right);

    let (_width_l, _width_r) = LineOffset::Center.widths(width);

    let (c_left, c_right) = if extra {
        (Rgb888::RED, Rgb888::GREEN)
    } else {
        (Rgb888::CSS_CORNFLOWER_BLUE, Rgb888::YELLOW)
    };

    // Add one to width so we get an extra iteration for the AA edge
    let wthr = (width + 1).pow(2) * (dx.pow(2) + dy.pow(2));
    let init_offset = dx + dy - (winit * sign);
    let mut tk = init_offset;
    // let mut tk: i32 = 0;

    // dbg!(wthr);

    // println!("===");

    // Perpendicular iteration
    while tk.pow(2) <= wthr {
        Pixel(point, c_left).draw(display)?;

        if error > threshold {
            point += step.major;
            error += e_minor;
            tk += 2 * dy;
        }

        error += e_major;
        point += step.minor;
        tk += 2 * dx;

        if tk.pow(2) > wthr {
            let fract = tk.pow(2) * 255 / (wthr / 2);
            let fract = fract.min(255 * 3 - 1);
            let fract = 255 - (fract % 255) as u32;

            let c = Rgb888::new(
                ((fract * c_left.r() as u32) / 255) as u8,
                ((fract * c_left.g() as u32) / 255) as u8,
                ((fract * c_left.b() as u32) / 255) as u8,
            );

            Pixel(point, c).draw(display)?;
        }
    }

    let mut point = Point::new(x0, y0);
    let mut error = einit * -sign;

    let mut tk = dx + dy + (winit * sign);

    while tk.pow(2) <= wthr {
        Pixel(point, c_right).draw(display)?;

        if error > threshold {
            point -= step.major;
            error += e_minor;
            tk += 2 * dy;
        }

        error += e_major;
        point -= step.minor;
        tk += 2 * dx;
    }

    Ok(())
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

    let mut p_error = 0;
    let mut error = 0;
    let mut point = start;

    let dx = delta.major.abs();
    let dy = delta.minor.abs();

    let threshold = dx - 2 * dy;
    let e_minor = -2 * dx;
    let e_major = 2 * dy;
    let length = dx + 1;

    let _greys = 255.0 / dx as f32;

    let skele_color = Rgb888::MAGENTA;

    for _i in 0..length {
        // let draw_skele = i % 2 == 0;
        let draw_skele = false;

        perpendicular(
            display, line, extents, point.x, point.y, delta, pstep, p_error, width, error, false,
        )?;

        if draw_skele {
            Pixel(point, skele_color).draw(display)?;
        }

        if error > threshold {
            point += step.minor;
            error += e_minor;

            if p_error >= threshold {
                if width > 1 {
                    perpendicular(
                        display,
                        line,
                        extents,
                        point.x,
                        point.y,
                        delta,
                        pstep,
                        p_error + e_minor + e_major,
                        width,
                        error,
                        true,
                    )?;

                    if draw_skele {
                        Pixel(point, skele_color).draw(display)?;
                    }
                }

                p_error += e_minor;
            }

            p_error += e_major;
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
