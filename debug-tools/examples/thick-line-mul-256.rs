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

    // Draw line using existing algorithm to check against
    // if extra {
    //     let mut line = line;

    //     // line.start.y += width * 2;
    //     // line.end.y += width * 2;

    //     line.into_styled(embedded_graphics::primitives::PrimitiveStyle::with_stroke(
    //         Rgb888::WHITE,
    //         width as u32,
    //     ))
    //     .draw(display)?;
    // }

    let non_mul_line = line;
    let non_mul_perpendicular_delta = line.perpendicular().delta();
    let seed_line = line.perpendicular();

    let parallel_is_y_major = line.delta().y.abs() >= line.delta().x.abs();

    let seed_is_y_major =
        non_mul_perpendicular_delta.y.abs() >= non_mul_perpendicular_delta.x.abs();

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

    let parallel_delta = mul_line.delta();

    let mul_delta = mul_line.delta();

    let mul_seed = {
        let mut line = seed_line;

        if seed_is_y_major {
            line.start.x *= 256;
            line.end.x *= 256;
        } else {
            line.start.y *= 256;
            line.end.y *= 256;
        }

        line
    };

    let mul_seed_delta = mul_seed.delta();

    let seed_step = Point::new(
        if mul_seed_delta.x >= 0 { 1 } else { -1 },
        if mul_seed_delta.y >= 0 { 1 } else { -1 },
    );

    let (thickness_majorminor, seed_line_delta, seed_step) = if seed_is_y_major {
        (
            MajorMinor::new(non_mul_perpendicular_delta.y, non_mul_perpendicular_delta.x),
            MajorMinor::new(mul_seed_delta.y, mul_seed_delta.x),
            MajorMinor::new(
                seed_step.y_axis(),
                Point::new((mul_seed_delta.x / mul_seed_delta.y).abs(), 0).component_mul(seed_step),
            ),
        )
    }
    // X-major line (i.e. X delta is longer than Y)
    else {
        (
            MajorMinor::new(non_mul_perpendicular_delta.x, non_mul_perpendicular_delta.y),
            MajorMinor::new(mul_seed_delta.x, mul_seed_delta.y),
            MajorMinor::new(
                seed_step.x_axis(),
                Point::new(0, (mul_seed_delta.y / mul_seed_delta.x).abs()).component_mul(seed_step),
            ),
        )
    };

    // ---

    let parallel_step = Point::new(
        if parallel_delta.x >= 0 { 1 } else { -1 },
        if parallel_delta.y >= 0 { 1 } else { -1 },
    );

    let (parallel_delta, parallel_step, parallel_step_full) = if parallel_is_y_major {
        (
            MajorMinor::new(parallel_delta.y.abs(), parallel_delta.x.abs()),
            MajorMinor::new(
                parallel_step.y_axis(),
                Point::new((mul_delta.x / mul_delta.y).abs(), 0).component_mul(parallel_step),
            ),
            MajorMinor::new(parallel_step.y_axis(), parallel_step.x_axis() * 256),
        )
    } else {
        (
            MajorMinor::new(parallel_delta.x.abs(), parallel_delta.y.abs()),
            MajorMinor::new(
                parallel_step.x_axis(),
                Point::new(0, (mul_delta.y / mul_delta.x).abs()).component_mul(parallel_step),
            ),
            MajorMinor::new(parallel_step.x_axis(), parallel_step.y_axis() * 256),
        )
    };

    // ---

    let dx = seed_line_delta.major.abs();
    let dy = seed_line_delta.minor.abs();

    // Using non-multiplied line delta otherwise thickness threshold runs into overflow issues (I
    // think? It ended up negative in testing)
    let thickness_dx = thickness_majorminor.major.abs();
    let thickness_dy = thickness_majorminor.minor.abs();

    // let threshold = dx - 2 * dy;
    // http://kt8216.unixcab.org/murphy/index.html calls e_minor E_diag, and e_major E_square
    let e_minor = -2 * dx;
    let e_major = 2 * dy;
    let mut seed_line_error = 2 * dy - dx;

    // Subtract 1 if using AA so 1px wide lines are _only_ drawn with AA - no solid fill
    let thickness_threshold =
        ((width - 1) * 2).pow(2) * non_mul_perpendicular_delta.length_squared();
    // Add the first line drawn to the thickness. If this is left at zero, an extra line will be
    // drawn as the lines are drawn before checking for thickness.
    let mut thickness_accumulator = 2 * thickness_dx;

    // // This fixes the phasing for parallel lines on the left side of the base line for the octants
    // // where the line perpendicular moves "away" from the line body.
    // let flip = if seed_line_step.minor == -parallel_step.major {
    //     -1
    // } else {
    //     1
    // };

    let swap_aa_direction = parallel_step_full.minor.x < 0 || parallel_step_full.minor.y < 0;

    let mut mul_point = mul_line.start;
    let mut seed_point = mul_seed.start;

    dbg!(seed_point, seed_step, seed_is_y_major);

    while thickness_accumulator.pow(2) <= thickness_threshold {
        let p = Point::new(
            if seed_is_y_major {
                seed_point.x >> 8
            } else {
                seed_point.x
            },
            if seed_is_y_major {
                seed_point.y
            } else {
                seed_point.y >> 8
            },
        );

        Pixel(p, Rgb888::RED).draw(display)?;

        // Move seed line in minor direction
        if seed_line_error > 0 {
            seed_line_error += e_minor;

            mul_point += parallel_step_full.major;
            seed_point += seed_step.minor;

            // parallel_line_2(
            //     mul_point,
            //     parallel_is_y_major,
            //     parallel_step,
            //     parallel_delta,
            //     Rgb888::CSS_AQUAMARINE,
            //     display,
            //     true,
            // )?;

            thickness_accumulator += 2 * thickness_dy;
        } else {
            // parallel_line_2(
            //     mul_point,
            //     parallel_is_y_major,
            //     parallel_step,
            //     parallel_delta,
            //     Rgb888::CSS_AQUAMARINE,
            //     display,
            //     false,
            // )?;
        }

        seed_line_error += e_major;
        thickness_accumulator += 2 * thickness_dx;
        seed_point += seed_step.major;
        mul_point += parallel_step_full.minor * -1;
    }

    // if extra {
    //     // Final AA line
    //     parallel_line_aa(
    //         mul_point,
    //         parallel_is_y_major,
    //         parallel_step,
    //         parallel_delta,
    //         // Rgb888::CSS_GOLDENROD,
    //         Rgb888::CSS_AQUAMARINE,
    //         swap_aa_direction,
    //         display,
    //     )?;

    //     // First AA line
    //     parallel_line_aa(
    //         mul_line.start + parallel_step_full.minor,
    //         parallel_is_y_major,
    //         parallel_step,
    //         parallel_delta,
    //         // Rgb888::CSS_GOLDENROD,
    //         Rgb888::CSS_AQUAMARINE,
    //         !swap_aa_direction,
    //         display,
    //     )?;
    // }

    // {
    //     let line = Line::new(mul_line.start, mul_point);

    //     let delta = line.delta();

    //     let step = Point::new(
    //         if delta.x >= 0 { 1 } else { -1 },
    //         if delta.y >= 0 { 1 } else { -1 },
    //     );

    //     let (delta, step) = if !seed_is_y_major {
    //         (
    //             MajorMinor::new(delta.y.abs(), delta.x.abs()),
    //             MajorMinor::new(
    //                 step.y_axis(),
    //                 Point::new((delta.x / delta.y).abs(), 0).component_mul(step),
    //             ),
    //         )
    //     } else {
    //         (
    //             MajorMinor::new(delta.x.abs(), delta.y.abs()),
    //             MajorMinor::new(
    //                 step.x_axis(),
    //                 Point::new(0, (delta.y / delta.x).abs()).component_mul(step),
    //             ),
    //         )
    //     };

    //     dbg!(delta, step, line, seed_is_y_major);

    //     parallel_line_2(
    //         line.start,
    //         !seed_is_y_major,
    //         step,
    //         delta,
    //         Rgb888::RED,
    //         display,
    //         false,
    //     )?;
    // }

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
    line_is_y_major: bool,
    step: MajorMinor<Point>,
    delta: MajorMinor<i32>,
    c: Rgb888,
    swap_aa_direction: bool,
    display: &mut impl DrawTarget<Color = Rgb888, Error = std::convert::Infallible>,
) -> Result<(), std::convert::Infallible> {
    let mut point = start;

    let dx = delta.major;
    let dy = delta.minor;

    let e_minor = -2 * dx;
    let e_major = 2 * dy;
    let length = dx;
    let mut error = 2 * dy - dx;

    // Blend colour for AA edge
    let background = Rgb888::BLACK;

    // FIXME: If line is exactly diagonal, no AA is performed. It should have a 50% edge.

    for _i in 0..length {
        let aa_colour = {
            let mul = (if line_is_y_major {
                point.x & 255
            } else {
                point.y & 255
            }) as u8;

            // Some octants need the AA direction to go the other way
            let mul = if swap_aa_direction { 255 - mul } else { mul };

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

        Pixel(aa_p, aa_colour).draw(display)?;

        if error > 0 {
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
    line_is_y_major: bool,
    step: MajorMinor<Point>,
    delta: MajorMinor<i32>,
    c: Rgb888,
    display: &mut impl DrawTarget<Color = Rgb888, Error = std::convert::Infallible>,
    extra: bool,
) -> Result<(), std::convert::Infallible> {
    let mut point = start;

    let dx = delta.major;
    let dy = delta.minor;

    let e_minor = -2 * dx;
    let e_major = 2 * dy;
    let length = dx;
    // Setting this to zero causes the first segment before the minor step to be too long
    let mut error = 2 * dy - dx;

    for _i in 0..length {
        let p = Point::new(
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
