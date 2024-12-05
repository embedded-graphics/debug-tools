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
    phase: i32,
) -> Result<(), std::convert::Infallible> {
    if width == 0 {
        return Ok(());
    }

    // // Draw line using existing algorithm to check against
    // if extra {
    //     let mut line = line;

    //     // line.start.y += width * 2;
    //     // line.end.y += width * 2;

    //     line.into_styled(embedded_graphics::primitives::PrimitiveStyle::with_stroke(
    //         Rgb888::WHITE,
    //         1,
    //     ))
    //     .draw(display)?;
    // }

    let original_parallel = line;
    let original_parallel_delta = original_parallel.delta();
    let original_seed = original_parallel.perpendicular();
    let original_seed_delta = original_seed.delta();
    let original_delta_majorminor =
        if original_parallel_delta.y.abs() >= original_parallel_delta.x.abs() {
            MajorMinor::new(original_parallel_delta.y, original_parallel_delta.x)
        } else {
            MajorMinor::new(original_parallel_delta.x, original_parallel_delta.y)
        };
    let original_seed_step = Point::new(
        if original_seed_delta.x >= 0 { 1 } else { -1 },
        if original_seed_delta.y >= 0 { 1 } else { -1 },
    );
    let original_seed_step_majorminor =
        if original_seed_delta.y.abs() >= original_seed_delta.x.abs() {
            MajorMinor::new(original_seed_step.y_axis(), original_seed_step.x_axis())
        } else {
            MajorMinor::new(original_seed_step.x_axis(), original_seed_step.y_axis())
        };

    let line = Line::new(line.start * 256, line.end * 256);

    let seed_line = line.perpendicular();
    let seed_delta = seed_line.delta();

    let parallel_delta = line.delta();

    let parallel_is_y_major = line.delta().y.abs() >= line.delta().x.abs();
    let seed_is_y_major = seed_delta.y.abs() >= seed_delta.x.abs();

    let seed_step = Point::new(
        if seed_delta.x >= 0 { 1 } else { -1 },
        if seed_delta.y >= 0 { 1 } else { -1 },
    ) * 256;

    let parallel_step = Point::new(
        if parallel_delta.x >= 0 { 1 } else { -1 },
        if parallel_delta.y >= 0 { 1 } else { -1 },
    ) * 256;

    // ---

    let seed_delta_majorminor = if seed_is_y_major {
        MajorMinor::new(seed_delta.y, seed_delta.x)
    } else {
        MajorMinor::new(seed_delta.x, seed_delta.y)
    };
    let parallel_delta_majorminor = if parallel_is_y_major {
        MajorMinor::new(parallel_delta.y / 256, parallel_delta.x)
    } else {
        MajorMinor::new(parallel_delta.x / 256, parallel_delta.y)
    };

    // Plain old boring multiplied by 256
    let seed_step_majorminor = if seed_is_y_major {
        MajorMinor::new(seed_step.y_axis(), seed_step.x_axis())
    } else {
        MajorMinor::new(seed_step.x_axis(), seed_step.y_axis())
    };
    let parallel_step_256_majorminor = if parallel_is_y_major {
        MajorMinor::new(parallel_step.y_axis(), parallel_step.x_axis())
    } else {
        MajorMinor::new(parallel_step.x_axis(), parallel_step.y_axis())
    };

    // Using line slope
    let seed_step_majorminor = if seed_is_y_major {
        MajorMinor::new(
            seed_step.y_axis(),
            Point::new(
                seed_delta
                    .x
                    .checked_div(original_seed_delta.x * seed_step.x.signum())
                    .unwrap_or(0),
                0,
            ),
        )
    } else {
        MajorMinor::new(
            seed_step.x_axis(),
            Point::new(
                0,
                seed_delta
                    .y
                    .checked_div(original_seed_delta.y * seed_step.y.signum())
                    .unwrap_or(0),
            ),
        )
    };
    let parallel_step_majorminor = if parallel_is_y_major {
        MajorMinor::new(
            parallel_step.y_axis(),
            Point::new(
                (original_parallel_delta.x * 256)
                    / (original_parallel_delta.y * parallel_step.y.signum()),
                0,
            ),
        )
    } else {
        MajorMinor::new(
            parallel_step.x_axis(),
            Point::new(
                0,
                (original_parallel_delta.y * 256)
                    / (original_parallel_delta.x * parallel_step.x.signum()),
            ),
        )
    };

    // ---

    let dx = seed_delta_majorminor.major.abs();
    let dy = seed_delta_majorminor.minor.abs();

    // http://kt8216.unixcab.org/murphy/index.html calls e_minor E_diag, and e_major E_square
    let e_minor = -2 * dx;
    let e_major = 2 * dy;

    let mut seed_line_error = 2 * dy - dx;
    let mut point = seed_line.start;
    let mut parallel_point = line.start;

    for i in 0..width {
        let p = point / 256;

        // assert_eq!(if seed_is_y_major { point.y } else { point.x } % 256, 0);

        Pixel(p, Rgb888::RED).draw(display)?;

        let aa_p = point / 256 - original_seed_step_majorminor.minor * 2;

        let aa_c = {
            let c = Rgb888::CSS_GOLDENROD;
            let background = Rgb888::BLACK;

            let mul = (if seed_is_y_major {
                point.x & 255
            } else {
                point.y & 255
            }) as u8;

            // // Some octants need the AA direction to go the other way
            // let mul = if swap_aa_direction { 255 - mul } else { mul };

            Rgb888::new(
                integer_lerp(c.r(), background.r(), mul),
                integer_lerp(c.g(), background.g(), mul),
                integer_lerp(c.b(), background.b(), mul),
            )
        };

        Pixel(aa_p, aa_c).draw(display)?;

        // Draw parallel line
        {
            let dx = parallel_delta_majorminor.major.abs();
            let dy = parallel_delta_majorminor.minor.abs();

            // http://kt8216.unixcab.org/murphy/index.html calls e_minor E_diag, and e_major E_square
            let e_minor = -2 * dx;
            let e_major = 2 * dy;

            let mut parallel_line_error = 2 * dy - dx;
            let mut point = parallel_point + parallel_step_majorminor.major * 2;

            for i in 0..original_delta_majorminor.major.abs() {
                let p = point / 256;

                Pixel(p, Rgb888::CSS_AQUAMARINE).draw(display)?;

                if parallel_line_error > 0 {
                    point += parallel_step_majorminor.minor;
                    parallel_line_error += e_minor;
                }

                point += parallel_step_majorminor.major;
                parallel_line_error += e_major;
            }
        }

        if seed_line_error > 0 {
            point += seed_step_majorminor.minor;
            parallel_point += parallel_step_256_majorminor.major;
            seed_line_error += e_minor;
        }

        point += seed_step_majorminor.major;
        parallel_point -= parallel_step_256_majorminor.minor;
        seed_line_error += e_major;
    }

    // A gap for debugging
    parallel_point += seed_step_majorminor.major;

    // Draw AA line
    {
        let dx = parallel_delta_majorminor.major.abs();
        let dy = parallel_delta_majorminor.minor.abs();

        // http://kt8216.unixcab.org/murphy/index.html calls e_minor E_diag, and e_major E_square
        let e_minor = -2 * dx;
        let e_major = 2 * dy;

        let mut parallel_line_error = 2 * dy - dx;
        let mut point = parallel_point + parallel_step_majorminor.major * 2;

        for i in 0..original_delta_majorminor.major.abs() {
            let aa_p = point / 256;

            let aa_c = {
                let c = Rgb888::CSS_GOLDENROD;
                let background = Rgb888::BLACK;

                let mul = (if parallel_is_y_major {
                    point.x & 255
                } else {
                    point.y & 255
                }) as u8;

                // // Some octants need the AA direction to go the other way
                // let mul = if swap_aa_direction { 255 - mul } else { mul };

                Rgb888::new(
                    integer_lerp(c.r(), background.r(), mul),
                    integer_lerp(c.g(), background.g(), mul),
                    integer_lerp(c.b(), background.b(), mul),
                )
            };

            Pixel(aa_p, aa_c).draw(display)?;

            if parallel_line_error > 0 {
                point += parallel_step_majorminor.minor;
                parallel_line_error += e_minor;
            }

            point += parallel_step_majorminor.major;
            parallel_line_error += e_major;
        }
    }

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

fn parallel_line_2(
    start: Point,
    line_is_y_major: bool,
    step: MajorMinor<Point>,
    delta: MajorMinor<i32>,
    c: Rgb888,
    display: &mut impl DrawTarget<Color = Rgb888, Error = std::convert::Infallible>,
    extra: bool,
    initial_error: i32,
) -> Result<(), std::convert::Infallible> {
    let mut point = start;

    point += step.major * 2;

    let dx = delta.major;
    let dy = delta.minor;

    let e_minor = -2 * dx;
    let e_major = 2 * dy;
    let length = dx;
    // Setting this to zero causes the first segment before the minor step to be too long
    // let mut error = 2 * dy - dx;
    let mut error = initial_error;

    for _i in 0..length {
        // let p = Point::new(
        //     if line_is_y_major {
        //         point.x >> 8
        //     } else {
        //         point.x
        //     },
        //     if line_is_y_major {
        //         point.y
        //     } else {
        //         point.y >> 8
        //     },
        // );

        let p = point;

        Pixel(p, c).draw(display)?;

        // Draws a pixel connecting a diagonal move into a solid stairstep-looking piece. This is
        // required for the additional diagonal move lines that are drawn when stepping in both the
        // major and minor directions in the seed line.
        if extra {
            let p = point + step.minor;

            let p = Point::new(
                if line_is_y_major { p.x >> 8 } else { p.x },
                if line_is_y_major { p.y } else { p.y >> 8 },
            );

            Pixel(p, c).draw(display)?;
        }

        if error > 0 {
            point += step.minor;
            error += e_minor;
        }

        point += step.major;
        error += e_major;
    }

    Ok(())
}

struct LineDebug {
    start: Point,
    end: Point,
    stroke_width: u32,
    phase: i32,
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
            stroke_width: 10,
            phase: 0,
            extra: true,
        }
    }

    fn parameters(&mut self) -> Vec<Parameter> {
        vec![
            Parameter::new("start", &mut self.start),
            Parameter::new("end", &mut self.end),
            Parameter::new("stroke", &mut self.stroke_width),
            Parameter::new("phase", &mut self.phase),
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

        thickline(
            display,
            Line::new(self.start, self.end),
            width,
            self.extra,
            self.phase,
        )?;

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
