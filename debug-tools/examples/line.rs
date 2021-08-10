use std::convert::TryFrom;

use embedded_graphics::{
    mock_display::MockDisplay,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Line, PrimitiveStyle},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use framework::prelude::*;

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
) -> Result<(), std::convert::Infallible> {
    let mut threshold = dx - 2 * dy;
    let mut E_diag = -2 * dx;
    let mut E_square = 2 * dy;
    let mut p = 0;
    let mut q = 0;

    let mut y = y0;
    let mut x = x0;
    let mut error = einit;
    let mut tk = dx + dy - winit;

    while tk <= width {
        Pixel(Point::new(x, y), Rgb565::RED).draw(display)?;

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

    let mut y = y0;
    let mut x = x0;
    let mut error = -einit;
    let mut tk = dx + dy + winit;

    while tk <= width {
        if p > 0 {
            Pixel(Point::new(x, y), Rgb565::GREEN).draw(display)?;
        }

        if error > threshold {
            x = x - xstep;
            error += E_diag;
            tk += 2 * dy;
        }

        error += E_square;
        y = y - ystep;
        tk += 2 * dx;
        p += 1;
    }

    // we need this for very thin lines
    if q == 0 && p < 2 {
        Pixel(Point::new(x0, y0), Rgb565::YELLOW).draw(display)?;
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
    let mut d = width * 2 * f32::sqrt((dx * dx + dy * dy) as f32) as i32;

    for p in 0..length {
        x_perpendicular(display, x, y, dx, dy, pxstep, pystep, p_error, d, error)?;
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
                    d,
                    error,
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

    while tk <= width {
        if p > 0 {
            Pixel(Point::new(x, y), Rgb565::GREEN).draw(display)?;
        }

        if error >= threshold {
            y = y - ystep;
            error += E_diag;
            tk += 2 * dx;
        }

        error += E_square;
        x = x - xstep;
        tk += 2 * dy;
        p += 1;
    }

    // we need this for very thin lines
    if q == 0 && p < 2 {
        Pixel(Point::new(x0, y0), Rgb565::YELLOW).draw(display)?;
    }

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
    let mut d = width * 2 * f32::sqrt((dx * dx + dy * dy) as f32) as i32;

    for p in 0..length {
        y_perpendicular(display, x, y, dx, dy, pxstep, pystep, p_error, d, error)?;

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
                    d,
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
    const DISPLAY_SIZE: Size = Size::new(256, 256);

    fn new() -> Self {
        let start = Point::new(
            Self::DISPLAY_SIZE.width as i32 / 2,
            Self::DISPLAY_SIZE.height as i32 / 2,
        );
        Self {
            start,
            end: start + Point::new(40, 50),
            stroke_width: 1,
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

        let width = 20;

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

        if dx > dy {
            x_varthick_line(display, x0, y0, dx, dy, xstep, ystep, pxstep, pystep, width)?;
        } else {
            y_varthick_line(display, x0, y0, dx, dy, xstep, ystep, pxstep, pystep, width)?;
        }

        // thin_octant1(display, x0, y0, dx, dy)?;
        // thick_octant1(display, x0, y0, x1, y1, 10)?;

        // FIXME: Overdraw
        // let mut mock_display = MockDisplay::new();
        // thick_octant1(&mut mock_display, x0, y0, x1, y1, 10).unwrap();

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
