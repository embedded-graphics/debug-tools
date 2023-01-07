use embedded_graphics::{
    geometry::PointExt, mock_display::MockDisplay, pixelcolor::Rgb888, prelude::*, primitives::Line,
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use framework::prelude::*;

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

fn thickline(
    display: &mut impl DrawTarget<Color = Rgb888, Error = std::convert::Infallible>,
    line: Line,
    width: i32,
    toggle: bool,
    toggle2: bool,
    last_offset: i32,
) -> Result<(), std::convert::Infallible> {
    if width == 0 {
        return Ok(());
    }

    let seed_line = line.perpendicular();

    let parallel_delta = line.end - line.start;
    let parallel_step = Point::new(
        if parallel_delta.x >= 0 { 1 } else { -1 },
        if parallel_delta.y >= 0 { 1 } else { -1 },
    );

    let mut point_left = line.start;
    let mut point_right = line.start;

    let seed_line_delta = seed_line.end - seed_line.start;

    let seed_line_direction = Point::new(
        if seed_line_delta.x >= 0 { 1 } else { -1 },
        if seed_line_delta.y >= 0 { 1 } else { -1 },
    );

    let (seed_line_delta, seed_line_step) = if seed_line_delta.y.abs() >= seed_line_delta.x.abs() {
        (
            MajorMinor::new(seed_line_delta.y, seed_line_delta.x),
            MajorMinor::new(seed_line_direction.y_axis(), seed_line_direction.x_axis()),
        )
    } else {
        (
            MajorMinor::new(seed_line_delta.x, seed_line_delta.y),
            MajorMinor::new(seed_line_direction.x_axis(), seed_line_direction.y_axis()),
        )
    };

    let (parallel_delta, parallel_step) = if parallel_delta.y.abs() >= parallel_delta.x.abs() {
        (
            MajorMinor::new(parallel_delta.y, parallel_delta.x),
            MajorMinor::new(parallel_step.y_axis(), parallel_step.x_axis()),
        )
    } else {
        (
            MajorMinor::new(parallel_delta.x, parallel_delta.y),
            MajorMinor::new(parallel_step.x_axis(), parallel_step.y_axis()),
        )
    };

    // Don't draw line skeleton twice
    point_right -= seed_line_step.major;

    let dx = seed_line_delta.major.abs();
    let dy = seed_line_delta.minor.abs();

    let threshold = dx - 2 * dy;
    let e_minor = -2 * dx;
    let e_major = 2 * dy;
    let mut seed_line_error = 0;
    let mut seed_line_error_right = e_major;
    // Perpendicular error or "phase"
    let mut parallel_error = 0;
    let mut parallel_error_right = 0;

    // This fixes the phasing for parallel lines on the left side of the base line for the octants
    // where the line perpendicular moves "away" from the line body.
    let original_flip = if seed_line_step.minor == -parallel_step.major {
        -1
    } else {
        1
    };

    // Subtract 1 if using AA so 1px wide lines are _only_ drawn with AA - no solid fill
    let thickness_threshold = ((width - 1) * 2).pow(2) * line.delta().length_squared();
    // Add the first line drawn to the thickness. If this is left at zero, an extra line will be
    // drawn as the lines are drawn before checking for thickness.
    let mut thickness_accumulator = 2 * dx;

    // Bias to one side of the line
    // TODO: The current extents() function needs to respect this too, as well as stroke offset
    let mut is_right = true;

    let mut right_side_aa_done = false;
    let mut left_side_aa_done = false;

    while thickness_accumulator.pow(2) <= thickness_threshold {
        let (point, inc, c, seed_line_error, parallel_error, flip) = if is_right {
            (
                &mut point_right,
                MajorMinor::new(-seed_line_step.major, -seed_line_step.minor),
                // Rgb888::CSS_DARK_GOLDENROD,
                Rgb888::CSS_SALMON,
                &mut seed_line_error_right,
                &mut parallel_error_right,
                // Fix phasing for parallel lines on the right hand side of the base line
                -original_flip,
            )
        } else {
            (
                &mut point_left,
                seed_line_step,
                Rgb888::CSS_SALMON,
                &mut seed_line_error,
                &mut parallel_error,
                original_flip,
            )
        };

        // Pixel(*point, c).draw(display)?;

        parallel_line(
            *point,
            line,
            parallel_step,
            parallel_delta,
            *parallel_error * flip,
            c,
            false,
            last_offset,
            display,
        )?;

        if *seed_line_error > threshold {
            *point += inc.minor;
            *seed_line_error += e_minor;
            thickness_accumulator += 2 * dy;

            if *parallel_error > threshold {
                if thickness_accumulator.pow(2) <= thickness_threshold {
                    // Pixel(*point, Rgb888::CYAN).draw(display)?;

                    parallel_line(
                        *point,
                        line,
                        parallel_step,
                        parallel_delta,
                        (*parallel_error + e_minor + e_major) * flip,
                        // Rgb888::CYAN,
                        c,
                        // If we're on the side of the base line where the perpendicular
                        // Bresenham steps "into" the thick line body, skip the first extra
                        // line point as it's on the wrong side of the perpendicular and leads
                        // to a jagged edge.
                        original_flip == -1 && !is_right || original_flip == 1 && is_right,
                        if original_flip == -1 && !is_right || original_flip == 1 && is_right {
                            0
                        } else {
                            // Because the opposite side's extra lines start one step into the thick
                            // line body, we must reduce its total length by 1 to prevent jagged
                            // edges on the end edge of the line.
                            -1
                        } + last_offset,
                        display,
                    )?;
                }

                // We're currently drawing an "extra" line. Special case: if the next step would be
                // line edge, draw an extra AA line and mark it as not needing to be drawn after
                // the main loop.
                if thickness_accumulator.pow(2) + 2 * dy > thickness_threshold {
                    if is_right {
                        right_side_aa_done = true;
                        println!("Right side AA inside loop");
                    } else {
                        left_side_aa_done = true;
                        println!("Left side AA inside loop");
                    }

                    if toggle {
                        parallel_line_aa(
                            *point,
                            line,
                            parallel_step,
                            (*parallel_error + e_minor + e_major) * flip,
                            c,
                            original_flip == -1 && !is_right || original_flip == 1 && is_right,
                            true,
                            if original_flip == -1 && !is_right || original_flip == 1 && is_right {
                                0
                            } else {
                                // Because the opposite side's extra lines start one step into the thick
                                // line body, we must reduce its total length by 1 to prevent jagged
                                // edges on the end edge of the line.
                                -1
                            } + last_offset,
                            display,
                        )?;
                    }
                }

                *parallel_error += e_minor;
            }

            *parallel_error += e_major;
        }

        *point += inc.major/* * 3*/;
        *seed_line_error += e_major;
        thickness_accumulator += 2 * dx;

        is_right = !is_right;
    }

    let flip = original_flip;

    if toggle {
        if !left_side_aa_done {
            parallel_line_aa(
                point_left,
                line,
                parallel_step,
                parallel_error * flip,
                Rgb888::CSS_SALMON,
                false,
                flip == -1,
                last_offset,
                display,
            )?;
        }

        if !right_side_aa_done {
            parallel_line_aa(
                point_right,
                line,
                parallel_step,
                parallel_error_right * -flip,
                Rgb888::CSS_SALMON,
                false,
                flip == 1,
                last_offset,
                display,
            )?;
        }
    }

    Ok(())
}

fn parallel_line_aa(
    start: Point,
    line: Line,
    step: MajorMinor<Point>,
    start_error: i32,
    c: Rgb888,
    skip_first: bool,
    invert: bool,
    mut last_offset: i32,
    display: &mut impl DrawTarget<Color = Rgb888, Error = std::convert::Infallible>,
) -> Result<(), std::convert::Infallible> {
    let mut point = start;

    let delta = line.delta();
    let delta = if delta.y.abs() >= delta.x.abs() {
        MajorMinor::new(delta.y, delta.x)
    } else {
        MajorMinor::new(delta.x, delta.y)
    };

    let dx = delta.major.abs();
    let dy = delta.minor.abs();

    let threshold = dx - 2 * dy;
    let e_minor = -2 * dx;
    let e_major = 2 * dy;
    let length = dx + 1;
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

    let grad_step = (dy * 255) / dx;
    // The gradient step is scaled based on how close to the diagonal we are. Horizontal/vertical
    // values are scaled by 1.0, trending to a scale of 0.5 for the diagonal. This allows diagonal
    // lines to have proper AA with each AA pixel having 0.5 brightness. Everything is multiplied
    // by 255 so we can do this with no floating point.
    let grad_step = (grad_step * 255) / (255 + grad_step);
    // We're in the center of a pixel at the start of the line, so start at half brightness. We'll
    // subtract one gradient step to prevent fireflies due to numerical errors in the second
    // iteration of the gradient loop.
    let mut bright_int: i32 = 127 - grad_step;

    for _i in 0..(length + last_offset) {
        let b = if invert { bright_int } else { 255 - bright_int };

        let c = Rgb888::new(
            ((b * c.r() as i32) / 255) as u8,
            ((b * c.g() as i32) / 255) as u8,
            ((b * c.b() as i32) / 255) as u8,
        );

        Pixel(point, c).draw(display)?;

        if error > threshold {
            point += step.minor;
            error += e_minor;
            bright_int = 0;
        }

        error += e_major;
        point += step.major;
        bright_int += grad_step;
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
    let mut point = start;

    // Pixel(point, c).draw(display)?;
    // return Ok(());

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
        Pixel(point, c).draw(display)?;

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
    toggle: bool,
    toggle2: bool,
    last_offset: i32,
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
            toggle: true,
            toggle2: true,
            last_offset: 0,
        }
    }

    fn parameters(&mut self) -> Vec<Parameter> {
        vec![
            Parameter::new("start", &mut self.start),
            Parameter::new("end", &mut self.end),
            Parameter::new("stroke", &mut self.stroke_width),
            Parameter::new("toggle", &mut self.toggle),
            Parameter::new("toggle2", &mut self.toggle2),
            Parameter::new("last offset", &mut self.last_offset),
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
            self.toggle,
            self.toggle2,
            self.last_offset,
        )?;

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
