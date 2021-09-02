use embedded_graphics::{
    mock_display::MockDisplay,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Line, PrimitiveStyle},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use framework::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq)]
enum LineSide {
    Left,
    Center,
    Right,
}

impl LineSide {
    fn widths(self, width: i32) -> (i32, i32) {
        match width {
            width => {
                match self {
                    Self::Left => ((width * 2).saturating_sub(1), 0),
                    Self::Center => {
                        let width = width.saturating_sub(1);

                        // Right-side bias for odd width lines. Move mod2 to first item to bias to
                        // the left.
                        (width, width + (width % 2))
                    }
                    Self::Right => ((width * 2).saturating_sub(1), 0),
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

fn perpendicular(
    display: &mut impl DrawTarget<Color = Rgb565, Error = std::convert::Infallible>,
    x0: i32,
    y0: i32,
    delta: MajorMinor<i32>,
    mut step: MajorMinor<Point>,
    einit: i32,
    width: i32,
    winit: i32,
    extra: bool,
) -> Result<(), std::convert::Infallible> {
    let mut point = Point::new(x0, y0);

    if width == 0 {
        return Ok(());
    }

    if width == 1 {
        return Pixel(point, Rgb565::YELLOW).draw(display);
    }

    let dx = delta.major;
    let dy = delta.minor;

    let mut sign = match (step.major, step.minor) {
        (Point { x: -1, y: 0 }, Point { x: 0, y: 1 }) => -1,
        (Point { x: 0, y: -1 }, Point { x: -1, y: 0 }) => -1,
        (Point { x: 1, y: 0 }, Point { x: 0, y: -1 }) => -1,
        (Point { x: 0, y: 1 }, Point { x: 1, y: 0 }) => -1,
        _ => 1,
    };

    if sign == -1 {
        step.major *= -1;
        step.minor *= -1;
    }

    let dx = dx.abs();
    let dy = dy.abs();

    let threshold = dx - 2 * dy;
    let e_minor = -2 * dx;
    let e_major = 2 * dy;

    let mut error = einit;
    let mut tk = -winit;

    let side = LineSide::Center;

    let (mut width_l, mut width_r) = side.widths(width);

    if sign == -1 {
        core::mem::swap(&mut width_l, &mut width_r);
    }

    if side == LineSide::Right {
        core::mem::swap(&mut width_l, &mut width_r);
    }

    let orig_width_l = width_l as f32;
    let orig_width_r = width_r as f32;

    let width_l = width_l.pow(2) * (dx * dx + dy * dy);
    let width_r = width_r.pow(2) * (dx * dx + dy * dy);

    let (c1, c2) = if extra {
        (Rgb565::RED, Rgb565::GREEN)
    } else {
        (Rgb565::CSS_CORNFLOWER_BLUE, Rgb565::YELLOW)
    };

    let (c1, c2) = (Rgb565::GREEN, Rgb565::GREEN);

    let mut distance = 0.0f32;

    let origin = Point::new(x0, y0);

    dbg!(orig_width_l);

    // while tk.pow(2) <= width_l + tk.pow(2) && width_l > 0 {
    while distance.floor() <= orig_width_l * 2.0 && width_l > 0 {
        println!("---");
        let thing = (tk.pow(2) - width_l) as f32 / width_l as f32;

        let thing = thing / orig_width_l;

        dbg!(thing, tk.pow(2), width_l);
        let fract = if tk.pow(2) > width_l {
            1.0 - thing
        } else {
            1.0
        };

        let fract = fract.max(0.2);

        Pixel(
            point,
            Rgb565::new(
                (c1.r() as f32 * fract) as u8,
                (c1.g() as f32 * fract) as u8,
                (c1.b() as f32 * fract) as u8,
            ),
        )
        .draw(display)?;

        if error >= threshold {
            point += step.major;
            error += e_minor;
            tk += 2 * dy;
        }

        error += e_major;
        point += step.minor;
        tk += 2 * dx;

        dbg!(distance);

        distance = {
            let delta = point - origin;

            f32::sqrt((delta.x.pow(2) + delta.y.pow(2)) as f32)
        };
    }

    println!("\n===========================\n");

    let mut point = Point::new(x0, y0);
    let mut error = -einit;
    let mut tk = winit;
    let mut p = 0;

    while tk.pow(2) <= width_r + tk.pow(2) && width_r > 0 {
        if p > 0 {
            let thing = (tk.pow(2) - width_l) as f32 / width_l as f32;
            let fract = if tk.pow(2) > width_l {
                1.0 - thing
            } else {
                1.0
            };

            Pixel(
                point,
                Rgb565::new(
                    (c1.r() as f32 * fract) as u8,
                    (c1.g() as f32 * fract) as u8,
                    (c1.b() as f32 * fract) as u8,
                ),
            )
            .draw(display)?;
        }

        if error > threshold {
            point -= step.major;
            error += e_minor;
            tk += 2 * dy;
        }

        error += e_major;
        point -= step.minor;
        tk += 2 * dx;
        p += 1;
    }

    Ok(())
}

fn thick_line(
    display: &mut impl DrawTarget<Color = Rgb565, Error = std::convert::Infallible>,
    start: Point,
    end: Point,
    width: i32,
) -> Result<(), std::convert::Infallible> {
    let (delta, step, pstep) = {
        let delta = end - start;

        let direction = Point::new(
            if delta.x >= 0 { 1 } else { -1 },
            if delta.y >= 0 { 1 } else { -1 },
        );

        let perp_direction = {
            let perp_delta = Point::new(delta.y, -delta.x);

            Point::new(
                if perp_delta.x >= 0 { 1 } else { -1 },
                if perp_delta.y >= 0 { 1 } else { -1 },
            )
        };

        // let delta = delta.abs();

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

    for _ in 0..length {
        perpendicular(
            display, point.x, point.y, delta, pstep, p_error, width, error, false,
        )?;

        // Pixel(point, Rgb565::WHITE).draw(display)?;

        if error > threshold {
            point += step.minor;
            error += e_minor;

            if p_error >= threshold {
                if width > 1 {
                    perpendicular(
                        display,
                        point.x,
                        point.y,
                        delta,
                        pstep,
                        p_error + e_minor + e_major,
                        width,
                        error,
                        true,
                    )?;

                    // Pixel(point, Rgb565::BLACK).draw(display)?;
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
    type Color = Rgb565;
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
        let Point { x: x0, y: y0 } = self.start;

        // let width = 2 * self.stroke_width as i32 * f32::sqrt((dx * dx + dy * dy) as f32) as i32;
        // let width = (self.stroke_width as i32).pow(2) * (dx * dx + dy * dy);
        let width = self.stroke_width as i32;

        let mut mock_display: MockDisplay<Rgb565> = MockDisplay::new();

        thick_line(display, self.start, self.end, width)?;

        Line::new(self.start, self.end)
            .into_styled(PrimitiveStyle::with_stroke(
                Rgb565::GREEN,
                self.stroke_width,
            ))
            .draw(&mut display.translated(Point::new(40, 40)))?;

        Ok(())
    }
}

fn main() {
    let settings = OutputSettingsBuilder::new().scale(5).build();
    let window = Window::new("Line debugger", &settings);

    LineDebug::run(window);
}
