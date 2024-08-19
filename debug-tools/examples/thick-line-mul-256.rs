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
    extra: bool,
) -> Result<(), std::convert::Infallible> {
    if width == 0 {
        return Ok(());
    }

    // Draw line using existing algorithm to check against
    // {
    //     let mut line = line;

    //     line.start.y += width * 2;
    //     line.end.y += width * 2;

    //     line.into_styled(PrimitiveStyle::with_stroke(Rgb888::WHITE, width as u32))
    //         .draw(display)?;
    // }

    let non_mul_line = line;
    let non_mul_perpendicular_delta = line.perpendicular().delta();
    let seed_line = line.perpendicular();

    let seed_is_y_major =
        non_mul_perpendicular_delta.y.abs() >= non_mul_perpendicular_delta.x.abs();

    let seed_line_delta = seed_line.delta();

    let seed_line_step = Point::new(
        if seed_line_delta.x >= 0 { 1 } else { -1 },
        if seed_line_delta.y >= 0 { 1 } else { -1 },
    );

    let (thickness_majorminor, seed_line_delta, mut seed_line_step) = if seed_is_y_major {
        (
            MajorMinor::new(non_mul_perpendicular_delta.y, non_mul_perpendicular_delta.x),
            MajorMinor::new(seed_line_delta.y, seed_line_delta.x),
            MajorMinor::new(seed_line_step.y_axis(), seed_line_step.x_axis()),
        )
    }
    // X-major line (i.e. X delta is longer than Y)
    else {
        (
            MajorMinor::new(non_mul_perpendicular_delta.x, non_mul_perpendicular_delta.y),
            MajorMinor::new(seed_line_delta.x, seed_line_delta.y),
            MajorMinor::new(seed_line_step.x_axis(), seed_line_step.y_axis()),
        )
    };

    // ---

    let parallel_is_y_major = line.delta().y.abs() >= line.delta().x.abs();

    // Using a block to isolate mutability
    let mul_line = {
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

    let mul_delta = mul_line.delta();

    let parallel_delta = mul_line.delta();

    let parallel_step = Point::new(
        if parallel_delta.x >= 0 { 1 } else { -1 },
        if parallel_delta.y >= 0 { 1 } else { -1 },
    );

    let (parallel_delta, parallel_step, parallel_step_full) = if parallel_is_y_major {
        (
            MajorMinor::new(parallel_delta.y, parallel_delta.x),
            MajorMinor::new(
                parallel_step.y_axis(),
                Point::new((mul_delta.x / mul_delta.y).abs(), 0).component_mul(parallel_step),
            ),
            MajorMinor::new(parallel_step.y_axis(), parallel_step.x_axis() * 256),
        )
    } else {
        (
            MajorMinor::new(parallel_delta.x, parallel_delta.y),
            MajorMinor::new(
                parallel_step.x_axis(),
                Point::new(0, (mul_delta.y / mul_delta.x).abs()).component_mul(parallel_step),
            ),
            MajorMinor::new(parallel_step.x_axis(), parallel_step.y_axis() * 256),
        )
    };

    // ---

    let slope = if parallel_is_y_major {
        mul_delta.x / mul_delta.y
    } else {
        mul_delta.y / mul_delta.x
    };

    let dx = seed_line_delta.major.abs();
    let dy = seed_line_delta.minor.abs();

    // Using non-multiplied line delta otherwise thickness threshold runs into overflow issues (I
    // think? It ended up negative in testing)
    let thickness_dx = thickness_majorminor.major.abs();
    let thickness_dy = thickness_majorminor.minor.abs();

    // Start error must be scaled the same as the major/minor errors used in `parallel_line()` to
    // set the starting error correctly.
    let parallel_dx = parallel_delta.major.abs();
    let parallel_dy = parallel_delta.minor.abs();

    let parallel_threshold = 2 * parallel_dy - parallel_dx;
    let parallel_e_minor = -2 * parallel_dx;
    let parallel_e_major = 2 * parallel_dy;

    // let threshold = dx - 2 * dy;
    // http://kt8216.unixcab.org/murphy/index.html calls e_minor E_diag, and e_major E_square
    let e_minor = -2 * dx;
    let e_major = 2 * dy;
    let mut seed_line_error = 2 * dy - dx;
    let mut parallel_error_left = 2 * parallel_dy - parallel_dx;
    // let mut parallel_error_left = 0i32;

    // Subtract 1 if using AA so 1px wide lines are _only_ drawn with AA - no solid fill
    let thickness_threshold =
        ((width - 1) * 2).pow(2) * non_mul_perpendicular_delta.length_squared();
    // Add the first line drawn to the thickness. If this is left at zero, an extra line will be
    // drawn as the lines are drawn before checking for thickness.
    let mut thickness_accumulator = 2 * thickness_dx;

    // This fixes the phasing for parallel lines on the left side of the base line for the octants
    // where the line perpendicular moves "away" from the line body.
    let flip = if seed_line_step.minor == -parallel_step.major {
        -1
    } else {
        1
    };

    let mut mul_point = mul_line.start;

    let flip = 1;

    // dbg!(
    //     seed_line_step,
    //     parallel_step,
    //     parallel_e_major,
    //     parallel_e_minor,
    //     flip
    // );

    // dbg!(
    //     seed_line_step,
    //     parallel_step,
    //     parallel_step_full,
    //     parallel_e_major,
    //     parallel_e_minor,
    //     flip,
    //     parallel_threshold
    // );

    let mut offset = 0;

    while thickness_accumulator.pow(2) <= thickness_threshold {
        parallel_line_2(
            mul_point,
            non_mul_line,
            parallel_step,
            parallel_delta,
            // if extra { parallel_error_left } else { 0 },
            // 2 * parallel_dy - parallel_dx,
            // if extra {
            //     parallel_error_left
            // } else {
            //     2 * parallel_dy - parallel_dx
            // },
            // parallel_error_left,
            2 * parallel_dy - parallel_dx,
            Rgb888::CSS_AQUAMARINE,
            false,
            0,
            display,
            false,
        )?;

        // Pixel(point, Rgb888::RED).draw(display)?;

        // Move seed line in minor direction
        if seed_line_error > 0 {
            seed_line_error += e_minor;

            if parallel_error_left > 0 {
                parallel_error_left += parallel_e_minor;

                mul_point += parallel_step_full.major * flip;
                // This makes things align properly, but it skews the seed line
                mul_point += parallel_step_full.minor * -flip;

                if thickness_accumulator.pow(2) <= thickness_threshold && extra {
                    parallel_line_2(
                        mul_point,
                        non_mul_line,
                        parallel_step,
                        parallel_delta,
                        2 * parallel_dy - parallel_dx,
                        Rgb888::CSS_GOLDENROD,
                        false,
                        0,
                        display,
                        true,
                    )?;
                }
            }

            thickness_accumulator += 2 * thickness_dy;
            parallel_error_left += parallel_e_major;
        }

        seed_line_error += e_major;
        thickness_accumulator += 2 * thickness_dx;

        mul_point += parallel_step_full.minor * -flip;
    }

    // Final AA line
    parallel_line_aa(
        mul_point,
        non_mul_line,
        parallel_step,
        parallel_delta,
        // Rgb888::CSS_GOLDENROD,
        Rgb888::CSS_AQUAMARINE,
        false,
        0,
        display,
    )?;

    Ok(())
}

/// Integer-only LERP with 8 bits of precision.
///
/// Thanks to <https://stackoverflow.com/a/34099335> for the inspiration.
fn integer_lerp(a: u8, b: u8, f: u8) -> u8 {
    let a = u16::from(a);
    let b = u16::from(b);
    let f = u16::from(f);

    let res = (a * (u16::from(u8::MAX) - f) + b * f) >> 8;

    res as u8
}

fn parallel_line_aa(
    start: Point,
    line: Line,
    step: MajorMinor<Point>,
    delta: MajorMinor<i32>,
    c: Rgb888,
    skip_first: bool,
    mut last_offset: i32,
    display: &mut impl DrawTarget<Color = Rgb888, Error = std::convert::Infallible>,
) -> Result<(), std::convert::Infallible> {
    let mut point = start;

    let line_is_y_major = line.delta().abs().y >= line.delta().abs().x;

    let dx = delta.major.abs();
    let dy = delta.minor.abs();

    let threshold = dx - 2 * dy;
    let e_minor = -2 * dx;
    let e_major = 2 * dy;
    // TODO: Skip first/last offset
    let mut length = dx + 1;
    let mut error = 0;

    let background = Rgb888::BLACK;

    for _i in 0..(length + last_offset) {
        let aa_colour = {
            let mul = (if line_is_y_major {
                point.x & 255
            } else {
                point.y & 255
            }) as u8;

            Rgb888::new(
                integer_lerp(c.r(), background.r(), mul),
                integer_lerp(c.g(), background.g(), mul),
                integer_lerp(c.b(), background.b(), mul),
            )
        };

        let aa_p = Point::new(
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

        // let aa_p = point;

        Pixel(aa_p, aa_colour).draw(display)?;

        if error > threshold {
            point += step.minor;
            error += e_minor;
        }

        error += e_major;
        point += step.major;
    }

    Ok(())
}

fn parallel_line_2(
    start: Point,
    line: Line,
    step: MajorMinor<Point>,
    delta: MajorMinor<i32>,
    start_error: i32,
    c: Rgb888,
    skip_first: bool,
    mut last_offset: i32,
    display: &mut impl DrawTarget<Color = Rgb888, Error = std::convert::Infallible>,
    extra: bool,
) -> Result<(), std::convert::Infallible> {
    let line_is_y_major = line.delta().abs().y >= line.delta().abs().x;

    let mut point = start;

    let dx = delta.major.abs();
    let dy = delta.minor.abs();

    // let threshold = 2 * dy - dx;
    let e_minor = -2 * dx;
    let e_major = 2 * dy;
    // TODO: Might need skip_first/last offset
    let mut length = dx + 1;
    // let mut error = if extra { 2 * dy - dx } else { start_error };
    let mut error = start_error;

    // dbg!(start_error, e_minor, e_major);

    // if skip_first {
    //     // Some of the length was consumed by this initial skip iteration. If this is omitted, the
    //     // line will be drawn 1px too long.
    //     last_offset -= 1;

    //     if error > 0 {
    //         point += step.minor;
    //         error += e_minor;
    //     }

    //     error += e_major;
    //     point += step.major;
    // }

    for _i in 0..(length + last_offset) {
        let aa_colour = c;

        let aa_p = Point::new(
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

        Pixel(aa_p, aa_colour).draw(display)?;

        if extra {
            let point = point + step.minor;

            let aa_p = Point::new(
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

            Pixel(aa_p, aa_colour).draw(display)?;
        }

        if error > 0 {
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
    extra: bool,
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
            stroke_width: 10,
            extra: true,
        }
    }

    fn parameters(&mut self) -> Vec<Parameter> {
        vec![
            Parameter::new("start", &mut self.start),
            Parameter::new("end", &mut self.end),
            Parameter::new("stroke", &mut self.stroke_width),
            Parameter::new("extra", &mut self.extra),
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

        thickline(display, Line::new(self.start, self.end), width, self.extra)?;

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
