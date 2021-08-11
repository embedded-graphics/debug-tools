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

fn x_perpendicular(
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
    extra: bool,
) -> Result<(), std::convert::Infallible> {
    if width == 0 {
        return Ok(());
    }

    if width == 1 {
        return Pixel(Point::new(x0, y0), Rgb565::YELLOW).draw(display);
    }

    let mut threshold = dx - 2 * dy;
    let mut E_diag = -2 * dx;
    let mut E_square = 2 * dy;
    let mut p = 0;
    let mut q = 0;

    let mut y = y0;
    let mut x = x0;
    let mut error = einit;
    // let mut tk = dx + dy - winit;

    // let mut tk = (dx + dy) / 2;
    // let mut tk = winit;

    let mut tk = -winit;
    let mut tk2 = winit;

    let mut y2 = y0;
    let mut x2 = x0;
    let mut error2 = -einit;
    // let mut tk2 = dx + dy + winit;

    // let width = width.saturating_sub(1);

    // let width_l = width;
    // // Put extras on right side. Move the %2 to width_l to place on left instead
    // let width_r = width + (width % 2);

    let side = LineSide::Center;

    let (width_l, width_r) = side.widths(width);

    let width_l = width_l.pow(2) * (dx * dx + dy * dy);
    let width_r = width_r.pow(2) * (dx * dx + dy * dy);

    let (c1, c2) = if extra {
        (Rgb565::RED, Rgb565::GREEN)
    } else {
        (Rgb565::CSS_CORNFLOWER_BLUE, Rgb565::YELLOW)
    };

    let mut swap = 1;

    // dbg!(width_l, width_r);

    while tk.pow(2) <= width_l && width_l > 0 {
        Pixel(Point::new(x, y), c1).draw(display)?;

        if error >= threshold {
            x += xstep;
            error += E_diag;
            tk += 2 * dy;
        }

        error += E_square;
        y += ystep;
        tk += 2 * dx;
        q += 1;
    }

    let mut y2 = y0;
    let mut x2 = x0;
    let mut error2 = -einit;
    let mut tk = winit;

    while tk.pow(2) <= width_r && width_r > 0 {
        if p > 0 && side == LineSide::Center {
            Pixel(Point::new(x2, y2), c2).draw(display)?;
        }

        if error2 > threshold {
            x2 -= xstep;
            error2 += E_diag;
            tk += 2 * dy;
        }

        error2 += E_square;
        y2 -= ystep;
        tk += 2 * dx;
        p += 1;
    }

    Ok(())
}

fn x_varthick_line(
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
    let mut threshold = dx - 2 * dy;
    let mut E_diag = -2 * dx;
    let mut E_square = 2 * dy;
    let mut length = dx + 1;

    for p in 0..length {
        x_perpendicular(
            display, x, y, dx, dy, pxstep, pystep, p_error, width, error, false,
        )?;
        if error >= threshold {
            y += ystep;
            error += E_diag;
            if p_error >= threshold {
                x_perpendicular(
                    display,
                    x,
                    y,
                    dx,
                    dy,
                    pxstep,
                    pystep,
                    p_error + E_diag + E_square,
                    width,
                    error,
                    true,
                )?;
                p_error += E_diag;
            }
            p_error += E_square;
        }
        error += E_square;
        x += xstep;
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
    let mut p = 0;
    let mut q = 0;
    let mut threshold = dy - 2 * dx;
    let mut E_diag = -2 * dy;
    let mut E_square = 2 * dx;

    let mut y = y0;
    let mut x = x0;
    let mut error = -einit;
    let mut tk = dx + dy + winit;

    while tk <= width {
        Pixel(Point::new(x, y), Rgb565::RED).draw(display)?;

        if error > threshold {
            y += ystep;
            error += E_diag;
            tk += 2 * dx;
        }

        error += E_square;
        x += xstep;
        tk += 2 * dy;
        q += 1;
    }

    let mut y = y0;
    let mut x = x0;
    let mut error = einit;
    let mut tk = dx + dy - winit;

    // while tk <= width {
    //     if p > 0 {
    //         Pixel(Point::new(x, y), Rgb565::GREEN).draw(display)?;
    //     }

    //     if error >= threshold {
    //         y -= ystep;
    //         error += E_diag;
    //         tk += 2 * dx;
    //     }

    //     error += E_square;
    //     x -= xstep;
    //     tk += 2 * dy;
    //     p += 1;
    // }

    // // we need this for very thin lines
    // if q == 0 && p < 2 {
    //     Pixel(Point::new(x0, y0), Rgb565::YELLOW).draw(display)?;
    // }

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
    let mut E_diag = -2 * dy;
    let mut E_square = 2 * dx;
    let mut length = dy + 1;

    for p in 0..length {
        y_perpendicular(display, x, y, dx, dy, pxstep, pystep, p_error, width, error)?;

        if error >= threshold {
            x += xstep;
            error += E_diag;
            if p_error >= threshold {
                y_perpendicular(
                    display,
                    x,
                    y,
                    dx,
                    dy,
                    pxstep,
                    pystep,
                    p_error + E_diag + E_square,
                    width,
                    error,
                )?;
                p_error += E_diag;
            }
            p_error += E_square;
        }
        error += E_square;
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
    // const DISPLAY_SIZE: Size = Size::new(256, 256);
    const DISPLAY_SIZE: Size = Size::new(64, 64);

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
        let Point { x: x1, y: y1 } = self.end;

        let pxstep;
        let pystep;
        let mut xch = 0; // whether left and right get switched.

        let mut dx = x1 - x0;
        let mut dy = y1 - y0;

        // let width = 2 * self.stroke_width as i32 * f32::sqrt((dx * dx + dy * dy) as f32) as i32;
        // let width = (self.stroke_width as i32).pow(2) * (dx * dx + dy * dy);
        let width = self.stroke_width as i32;

        let mut xstep = 1;
        let mut ystep = 1;

        if dx < 0 {
            dx = -dx;
            xstep = -1;
        }
        if dy < 0 {
            dy = -dy;
            ystep = -1;
        }

        if dx == 0 {
            xstep = 0;
        }
        if dy == 0 {
            ystep = 0;
        }

        match (xstep, ystep) {
            (-1, -1) => {
                pystep = -1;
                pxstep = 1;
                xch = 1;
            }
            (-1, 0) => {
                pystep = -1;
                pxstep = 0;
                xch = 1;
            }
            (-1, 1) => {
                pystep = 1;
                pxstep = 1;
            }
            (0, -1) => {
                pystep = 0;
                pxstep = -1;
            }
            (0, 0) => {
                pystep = 0;
                pxstep = 0;
            }
            (0, 1) => {
                pystep = 0;
                pxstep = 1;
            }
            (1, -1) => {
                pystep = -1;
                pxstep = -1;
            }
            (1, 0) => {
                pystep = -1;
                pxstep = 0;
            }
            (1, 1) => {
                pystep = 1;
                pxstep = -1;
                xch = 1;
            }
            _ => unreachable!(),
        }

        // TODO: xch or swap_sides

        let mut mock_display = MockDisplay::new();
        // mock_display.set_allow_out_of_bounds_drawing(true);

        if dx > dy {
            x_varthick_line(display, x0, y0, dx, dy, xstep, ystep, pxstep, pystep, width)?;
            x_varthick_line(
                &mut mock_display,
                x0,
                y0,
                dx,
                dy,
                xstep,
                ystep,
                pxstep,
                pystep,
                width,
            )?;
        } else {
            y_varthick_line(display, x0, y0, dx, dy, xstep, ystep, pxstep, pystep, width)?;
            y_varthick_line(
                &mut mock_display,
                x0,
                y0,
                dx,
                dy,
                xstep,
                ystep,
                pxstep,
                pystep,
                width,
            )?;
        }

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
