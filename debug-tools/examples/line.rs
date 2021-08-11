use std::convert::TryFrom;

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
            // 0 => (0, 0),
            width => {
                match self {
                    Self::Left => ((width * 2).saturating_sub(1), 0),
                    Self::Center => {
                        let width = width.saturating_sub(1);

                        // Right-side bias for odd width lines. Move mod2 to first item to bias to
                        // the left.
                        (width, width + (width % 2))
                    }
                    Self::Right => (0, (width * 2).saturating_sub(1)),
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

fn x_perpendicular(
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

    dbg!(step.major, step.minor);

    let sign = match (step.major, step.minor) {
        (Point { x: -1, y: 0 }, Point { x: 0, y: 1 }) => -1,
        (Point { x: 0, y: -1 }, Point { x: -1, y: 0 }) => -1,
        (Point { x: 1, y: 0 }, Point { x: 0, y: -1 }) => -1,
        (Point { x: 0, y: 1 }, Point { x: 1, y: 0 }) => -1,
        _ => 1,
    };

    step.major *= sign;
    step.minor *= sign;

    let dx = dx.abs();
    let dy = dy.abs();

    let threshold = dx - 2 * dy;
    let e_minor = -2 * dx;
    let e_major = 2 * dy;
    let mut p = 0;
    let mut q = 0;

    // let mut y = y0;
    // let mut x = x0;

    // Swap signs in some conditions
    let mut error = einit;
    let mut tk = -winit;

    let side = LineSide::Center;

    let (width_l, width_r) = side.widths(width);

    let width_l = width_l.pow(2) * (dx * dx + dy * dy);
    let width_r = width_r.pow(2) * (dx * dx + dy * dy);

    let (c1, c2) = if extra {
        (Rgb565::RED, Rgb565::GREEN)
    } else {
        (Rgb565::CSS_CORNFLOWER_BLUE, Rgb565::YELLOW)
    };

    // dbg!(width_l, width_r);

    while tk.pow(2) <= width_l && width_l > 0 {
        Pixel(point, c1).draw(display)?;

        if error >= threshold {
            point += step.major;
            error += e_minor;
            tk += 2 * dy;
        }

        error += e_major;
        point += step.minor;
        tk += 2 * dx;
        q += 1;
    }

    // let mut y2 = y0;
    // let mut x2 = x0;
    let mut point = Point::new(x0, y0);
    let mut error2 = -einit;
    let mut tk = winit;

    while tk.pow(2) <= width_r && width_r > 0 {
        if p > 0 && side == LineSide::Center {
            Pixel(point, c2).draw(display)?;
        }

        if error2 > threshold {
            point -= step.major;
            error2 += e_minor;
            tk += 2 * dy;
        }

        error2 += e_major;
        point -= step.minor;
        tk += 2 * dx;
        p += 1;
    }

    Ok(())
}

fn x_varthick_line(
    display: &mut impl DrawTarget<Color = Rgb565, Error = std::convert::Infallible>,
    x0: i32,
    y0: i32,
    delta: MajorMinor<i32>,
    step: MajorMinor<Point>,
    pstep: MajorMinor<Point>,
    width: i32,
) -> Result<(), std::convert::Infallible> {
    let mut p_error = 0;
    let mut error = 0;
    // let mut y = y0;
    // let mut x = x0;

    let mut point = Point::new(x0, y0);

    let dx = delta.major.abs();
    let dy = delta.minor.abs();

    let threshold = dx - 2 * dy;
    let e_minor = -2 * dx;
    let e_major = 2 * dy;
    let length = dx + 1;

    for p in 0..length {
        x_perpendicular(
            display, point.x, point.y, delta, pstep, p_error, width, error, false,
        )?;
        if error >= threshold {
            // y += ystep;
            point += step.minor;
            error += e_minor;
            if p_error >= threshold {
                x_perpendicular(
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
                p_error += e_minor;
            }
            p_error += e_major;
        }
        error += e_major;
        // x += xstep;
        point += step.major;
    }

    Ok(())
}

/***********************************************************************
 *                                                                     *
 *                            Y BASED LINES                            *
 *                                                                     *
 ***********************************************************************/

fn y_perpendicular(
    display: &mut impl DrawTarget<Color = Rgb565, Error = std::convert::Infallible>,
    x0: i32,
    y0: i32,
    dx: i32,
    dy: i32,
    xstep: i32,
    ystep: i32,
    einit: i32,
    width: i32,
    winit: i32,
) -> Result<(), std::convert::Infallible> {
    let p = 0;
    let mut q = 0;
    let threshold = dy - 2 * dx;
    let e_minor = -2 * dy;
    let e_major = 2 * dx;

    let mut y = y0;
    let mut x = x0;
    let mut error = -einit;
    let mut tk = dx + dy + winit;

    while tk <= width {
        Pixel(Point::new(x, y), Rgb565::RED).draw(display)?;

        if error > threshold {
            y += ystep;
            error += e_minor;
            tk += 2 * dx;
        }

        error += e_major;
        x += xstep;
        tk += 2 * dy;
        q += 1;
    }

    let y = y0;
    let x = x0;
    let error = einit;
    let tk = dx + dy - winit;

    Ok(())
}

fn y_varthick_line(
    display: &mut impl DrawTarget<Color = Rgb565, Error = std::convert::Infallible>,
    x0: i32,
    y0: i32,
    dx: i32,
    dy: i32,
    xstep: i32,
    ystep: i32,
    pxstep: i32,
    pystep: i32,
    width: i32,
) -> Result<(), std::convert::Infallible> {
    let mut p_error = 0;
    let mut error = 0;
    let mut y = y0;
    let mut x = x0;
    let mut threshold = dy - 2 * dx;
    let mut e_minor = -2 * dy;
    let mut e_major = 2 * dx;
    let mut length = dy + 1;

    for p in 0..length {
        y_perpendicular(display, x, y, dx, dy, pxstep, pystep, p_error, width, error)?;

        if error >= threshold {
            x += xstep;
            error += e_minor;
            if p_error >= threshold {
                y_perpendicular(
                    display,
                    x,
                    y,
                    dx,
                    dy,
                    pxstep,
                    pystep,
                    p_error + e_minor + e_major,
                    width,
                    error,
                )?;
                p_error += e_minor;
            }
            p_error += e_major;
        }
        error += e_major;
        y += ystep;
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
    const DISPLAY_SIZE: Size = Size::new(256, 256);
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
        // let Point { x: x1, y: y1 } = self.end;

        // // let pxstep;
        // // let pystep;
        // // let mut xch = 0; // whether left and right get switched.

        // let mut dx = x1 - x0;
        // let mut dy = y1 - y0;

        // let width = 2 * self.stroke_width as i32 * f32::sqrt((dx * dx + dy * dy) as f32) as i32;
        // let width = (self.stroke_width as i32).pow(2) * (dx * dx + dy * dy);
        let width = self.stroke_width as i32;

        // let mut xstep = 1;
        // let mut ystep = 1;

        // if dx < 0 {
        //     dx = -dx;
        //     xstep = -1;
        // }
        // if dy < 0 {
        //     dy = -dy;
        //     ystep = -1;
        // }

        // if dx == 0 {
        //     xstep = 0;
        // }
        // if dy == 0 {
        //     ystep = 0;
        // }

        // match (xstep, ystep) {
        //     (-1, -1) => {
        //         pystep = -1;
        //         pxstep = 1;
        //         xch = 1;
        //     }
        //     (-1, 0) => {
        //         pystep = -1;
        //         pxstep = 0;
        //         xch = 1;
        //     }
        //     (-1, 1) => {
        //         pystep = 1;
        //         pxstep = 1;
        //     }
        //     (0, -1) => {
        //         pystep = 0;
        //         pxstep = -1;
        //     }
        //     (0, 0) => {
        //         pystep = 0;
        //         pxstep = 0;
        //     }
        //     (0, 1) => {
        //         pystep = 0;
        //         pxstep = 1;
        //     }
        //     (1, -1) => {
        //         pystep = -1;
        //         pxstep = -1;
        //     }
        //     (1, 0) => {
        //         pystep = -1;
        //         pxstep = 0;
        //     }
        //     (1, 1) => {
        //         pystep = 1;
        //         pxstep = -1;
        //         xch = 1;
        //     }
        //     _ => unreachable!(),
        // }

        // // TODO: xch or swap_sides

        let (delta, step, pstep) = {
            let delta = self.end - self.start;

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

        let mut mock_display: MockDisplay<Rgb565> = MockDisplay::new();
        // mock_display.set_allow_out_of_bounds_drawing(true);

        // if delta.x > delta.y {
        x_varthick_line(display, x0, y0, delta, step, pstep, width)?;
        // x_varthick_line(
        //     &mut mock_display,
        //     x0,
        //     y0,
        //     dx,
        //     dy,
        //     xstep,
        //     ystep,
        //     pxstep,
        //     pystep,
        //     width,
        // )?;
        // } else {
        //     y_varthick_line(display, x0, y0, dx, dy, xstep, ystep, pxstep, pystep, width)?;
        //     // y_varthick_line(
        //     //     &mut mock_display,
        //     //     x0,
        //     //     y0,
        //     //     dx,
        //     //     dy,
        //     //     xstep,
        //     //     ystep,
        //     //     pxstep,
        //     //     pystep,
        //     //     width,
        //     // )?;
        // }

        // thin_octant1(display, x0, y0, dx, dy)?;
        // thick_octant1(display, x0, y0, x1, y1, 10)?;

        Ok(())

        // Line::new(self.start, self.end)
        //     .into_styled(PrimitiveStyle::with_stroke(
        //         Rgb565::GREEN,
        //         self.stroke_width,
        //     ))
        //     .draw(display)
    }
}

fn main() {
    let settings = OutputSettingsBuilder::new().scale(3).build();
    let window = Window::new("Line debugger", &settings);

    LineDebug::run(window);
}
