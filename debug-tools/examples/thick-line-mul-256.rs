use embedded_graphics::{
    geometry::PointExt, mock_display::MockDisplay, pixelcolor::Rgb888, prelude::*, primitives::Line,
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

fn thickline(
    display: &mut impl DrawTarget<Color = Rgb888, Error = std::convert::Infallible>,
    line: Line,
    width: i32,
) -> Result<(), std::convert::Infallible> {
    if width == 0 {
        return Ok(());
    }

    let non_mul_line = line;
    let non_mul_perpendicular_delta = line.perpendicular().delta();
    let mut seed_line = line.perpendicular();

    let non_mul_delta = seed_line.delta();

    let seed_is_y_major = non_mul_delta.y.abs() >= non_mul_delta.x.abs();

    // Multiply minor direction by 256 so we get AA resolution in lower 8 bits. Not used for seed
    // line directly, but is used to scale initial error for each parallel line.
    if seed_is_y_major {
        seed_line.start.x *= 256;
        seed_line.end.x *= 256;
    } else {
        seed_line.start.y *= 256;
        seed_line.end.y *= 256;
    }

    let seed_line_delta = seed_line.delta();

    let seed_line_step = Point::new(
        if seed_line_delta.x >= 0 { 1 } else { -1 },
        if seed_line_delta.y >= 0 { 1 } else { -1 },
    );

    let (thickness_majorminor, seed_line_delta, seed_line_step) = if seed_is_y_major {
        (
            MajorMinor::new(non_mul_perpendicular_delta.y, non_mul_perpendicular_delta.x),
            MajorMinor::new(seed_line_delta.y, seed_line_delta.x),
            // MajorMinor::new(seed_line_step.y_axis(), seed_line_step.x_axis()),
            MajorMinor::new(
                seed_line_step.y_axis(),
                Point::new((seed_line_delta.x / seed_line_delta.y).abs(), 0)
                    .component_mul(seed_line_step),
            ),
        )
    }
    // X-major line (i.e. X delta is longer than Y)
    else {
        (
            MajorMinor::new(non_mul_perpendicular_delta.x, non_mul_perpendicular_delta.y),
            MajorMinor::new(seed_line_delta.x, seed_line_delta.y),
            // MajorMinor::new(seed_line_step.x_axis(), seed_line_step.y_axis()),
            MajorMinor::new(
                seed_line_step.x_axis(),
                Point::new(0, (seed_line_delta.y / seed_line_delta.x).abs())
                    .component_mul(seed_line_step),
            ),
        )
    };

    // ---

    let parallel_is_y_major = line.delta().y.abs() >= line.delta().x.abs();

    // Using a block to isolate mutability
    let line = {
        let mut line = line;

        // Multiply minor direction by 256 so we get AA resolution in lower 8 bits
        if parallel_is_y_major {
            line.start.x *= 256;
            line.end.x *= 256;
        } else {
            line.start.y *= 256;
            line.end.y *= 256;
        }

        line
    };

    let parallel_delta = line.delta();

    let parallel_step = Point::new(
        if parallel_delta.x >= 0 { 1 } else { -1 },
        if parallel_delta.y >= 0 { 1 } else { -1 },
    );

    let (parallel_delta, parallel_step) = if parallel_is_y_major {
        (
            MajorMinor::new(parallel_delta.y, parallel_delta.x),
            // MajorMinor::new(parallel_step.y_axis(), parallel_step.x_axis()),
            MajorMinor::new(
                parallel_step.y_axis(),
                Point::new((parallel_delta.x / parallel_delta.y).abs(), 0)
                    .component_mul(parallel_step),
            ),
        )
    } else {
        (
            MajorMinor::new(parallel_delta.x, parallel_delta.y),
            // MajorMinor::new(parallel_step.x_axis(), parallel_step.y_axis()),
            MajorMinor::new(
                parallel_step.x_axis(),
                Point::new(0, (parallel_delta.y / parallel_delta.x).abs())
                    .component_mul(parallel_step),
            ),
        )
    };

    // ---

    let mut point = seed_line.start;

    let dx = seed_line_delta.major.abs();
    let dy = seed_line_delta.minor.abs();

    // Using non-multiplied line delta otherwise thickness threshold runs into overflow issues (I
    // think? It ended up negative in testing)
    let thickness_dx = thickness_majorminor.major.abs();
    let thickness_dy = thickness_majorminor.minor.abs();

    let threshold = dx - 2 * dy;
    // http://kt8216.unixcab.org/murphy/index.html calls e_minor E_diag, and e_major E_square
    let e_minor = -2 * dx;
    let e_major = 2 * dy;
    let mut seed_line_error = 0;
    let mut parallel_error_left = 0;

    // Subtract 1 if using AA so 1px wide lines are _only_ drawn with AA - no solid fill
    let thickness_threshold =
        ((width - 1) * 2).pow(2) * non_mul_perpendicular_delta.length_squared();
    // Add the first line drawn to the thickness. If this is left at zero, an extra line will be
    // drawn as the lines are drawn before checking for thickness.
    let mut thickness_accumulator = 2 * thickness_dx;

    // println!(
    //     "thresh {} thick thresh {} e_minor {} e_major {} dx {} dy {} y major {}",
    //     threshold, thickness_threshold, e_minor, e_major, dx, dy, y_major
    // );

    while thickness_accumulator.pow(2) <= thickness_threshold {
        // println!("error {} point {}", seed_line_error, point);

        let c = Rgb888::CSS_FOREST_GREEN;

        let p = Point::new(
            if seed_is_y_major {
                point.x >> 8
            } else {
                point.x
            },
            if seed_is_y_major {
                point.y
            } else {
                point.y >> 8
            },
        );

        Pixel(p, c).draw(display)?;

        parallel_line(
            point,
            non_mul_line,
            parallel_step,
            parallel_delta,
            parallel_error_left,
            Rgb888::CSS_AQUAMARINE,
            false,
            0,
            display,
        )?;

        // We seem to hit the threshold too early and end up with a 45 degree line everywhere.
        if seed_line_error > threshold {
            point += seed_line_step.minor;
            seed_line_error += e_minor;
            thickness_accumulator += 2 * thickness_dy;

            // Attempted fixes:
            // - Threshold can be negative. .abs() doesn't fix the start error value
            if parallel_error_left > threshold {
                parallel_error_left += e_minor;
            }

            parallel_error_left += e_major;
        }

        point += seed_line_step.major * 2;
        seed_line_error += e_major;
        thickness_accumulator += 2 * thickness_dx;
    }

    // parallel_line(
    //     point,
    //     non_mul_line,
    //     parallel_step,
    //     parallel_delta,
    //     parallel_error_left,
    //     Rgb888::CSS_SALMON,
    //     false,
    //     0,
    //     display,
    // )?;

    Ok(())
}

fn parallel_line_aa(
    start: Point,
    line: Line,
    step: MajorMinor<Point>,
    delta: MajorMinor<i32>,
    start_error: i32,
    c: Rgb888,
    skip_first: bool,
    invert: bool,
    mut last_offset: i32,
    display: &mut impl DrawTarget<Color = Rgb888, Error = std::convert::Infallible>,
) -> Result<(), std::convert::Infallible> {
    let mut point = start;

    let y_major = line.delta().y >= line.delta().x;

    let dx = delta.major.abs();
    let dy = delta.minor.abs();

    let threshold = dx - 2 * dy;
    let e_minor = -2 * dx;
    let e_major = 2 * dy;
    let mut length = dx + 1;
    let mut error = start_error;

    if skip_first {
        // Some of the length was consumed by this initial skip iteration. If this is omitted, the
        // line will be drawn 1px too long.
        last_offset -= 1;

        if error > threshold {
            point += step.minor;
            error += e_minor;
        }

        error += e_major;
        point += step.major;
    }

    for _i in 0..(length + last_offset) {
        // https://computergraphics.stackexchange.com/a/10675
        let draw_p = Point::new(point.x, (point.y >> 8) - (line.delta().y).signum());

        Pixel(draw_p, Rgb888::CYAN).draw(display)?;

        {
            let aa_colour = {
                let mul = (if y_major {
                    point.x & 255
                } else {
                    point.y & 255
                }) as u8;

                Rgb888::new(
                    // TODO: Proper colour blend
                    // (c.r() as f32 * (1.0 - mul as f32 / 255.0)) as u8,
                    // (c.g() as f32 * (1.0 - mul as f32 / 255.0)) as u8,
                    // (c.b() as f32 * (1.0 - mul as f32 / 255.0)) as u8,
                    255 - mul,
                    255 - mul,
                    255 - mul,
                )
            };

            let aa_p = Point::new(
                if y_major {
                    (point.x >> 8) - (line.delta().x).signum() * 2
                } else {
                    point.x
                },
                if y_major {
                    point.y
                } else {
                    (point.y >> 8) - (line.delta().y).signum() * 2
                },
            );

            Pixel(aa_p, aa_colour).draw(display)?;
        }

        // Doesn't work: mathematical distance from ideal line using line_point_distance(). Not
        // quite sure why but we don't get a smooth increase over the length of the line.

        if error > threshold {
            point += step.minor;
            error += e_minor;
        }

        error += e_major;
        point += step.major;
    }

    Ok(())
}

fn parallel_line(
    start: Point,
    line: Line,
    step: MajorMinor<Point>,
    delta: MajorMinor<i32>,
    start_error: i32,
    c: Rgb888,
    skip_first: bool,
    mut last_offset: i32,
    display: &mut impl DrawTarget<Color = Rgb888, Error = std::convert::Infallible>,
) -> Result<(), std::convert::Infallible> {
    dbg!(start, step);

    let line_is_y_major = line.delta().abs().y >= line.delta().abs().x;

    // Perpendicular axis is scaled by 256. We need to swap the scaled axes to account for this.
    let mut point = if line_is_y_major {
        Point::new(start.x * 256, start.y >> 8)
    } else {
        Point::new(start.x >> 8, start.y * 256)
    };

    let dx = delta.major.abs();
    let dy = delta.minor.abs();

    let threshold = dx - 2 * dy;
    let e_minor = -2 * dx;
    let e_major = 2 * dy;
    let mut length = dx + 1;
    let mut error = start_error;

    if skip_first {
        // Some of the length was consumed by this initial skip iteration. If this is omitted, the
        // line will be drawn 1px too long.
        last_offset -= 1;

        if error > threshold {
            point += step.minor;
            error += e_minor;
        }

        error += e_major;
        point += step.major;
    }

    for _i in 0..(length + last_offset) {
        // https://computergraphics.stackexchange.com/a/10675
        let draw_p = Point::new(
            if line_is_y_major {
                point.x >> 8
            } else {
                point.x
            },
            if line_is_y_major {
                point.y
            } else {
                point.y >> 8
            },
        );

        // dbg!(draw_p, step);

        Pixel(draw_p, c).draw(display)?;

        if error > threshold {
            point += step.minor;
            error += e_minor;
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
            start: end - Point::new(80, 35),
            end,
            // end: start + Point::new(100, 0),
            stroke_width: 20,
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

        // l.into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, width as u32))
        //     .draw(display)?;

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
