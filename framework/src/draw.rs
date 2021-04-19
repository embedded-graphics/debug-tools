use std::{convert::Infallible, marker::PhantomData, ops::Range};

use embedded_graphics::{
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle},
};
use embedded_graphics_simulator::SimulatorDisplay;

/// Draws a cross around a point.
pub fn point<D>(p: Point, color: D::Color, target: &mut D)
where
    D: DrawTarget<Error = Infallible>,
{
    Circle::with_center(p, 3)
        .into_styled(PrimitiveStyle::with_stroke(color, 1))
        .draw(target)
        .unwrap();
}

/// Draws the bounding box of a drawable.
///
/// This method draws two bounding boxes around the drawable. The bounding box returned by the
/// `Drawable` impl is drawn in gray. The second bounding box is determined by calculating the
/// minimum and maximum coordinates of all drawn pixels. For non transparent strokes both bounding
/// boxes should have the same size and only the gray bounding box should be visible.
pub fn bounding_box<T, C>(drawable: &T, display: &mut SimulatorDisplay<C>)
where
    T: Drawable + Dimensions,
    C: PixelColor + WebColors,
{
    // Determine actual bounding box
    let mut bb_target = BoundingBoxDrawTarget::new();
    drawable.draw(&mut bb_target).unwrap();

    bb_target
        .bounding_box
        .into_styled(PrimitiveStyle::with_stroke(C::CSS_TOMATO, 1))
        .draw(display)
        .unwrap();

    drawable
        .bounding_box()
        .into_styled(PrimitiveStyle::with_stroke(C::CSS_DIM_GRAY, 1))
        .draw(display)
        .unwrap();
}

#[derive(Debug)]
struct BoundingBoxDrawTarget<C> {
    bounding_box: Rectangle,
    color_type: PhantomData<C>,
}

impl<C> BoundingBoxDrawTarget<C> {
    fn new() -> Self {
        Self {
            bounding_box: Rectangle::zero(),
            color_type: PhantomData,
        }
    }
}

impl<C: PixelColor> DrawTarget for BoundingBoxDrawTarget<C> {
    type Color = C;
    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let mut columns = self.bounding_box.columns();
        let mut rows = self.bounding_box.rows();

        for Pixel(p, _c) in pixels {
            extend_range(&mut columns, p.x);
            extend_range(&mut rows, p.y);
        }

        self.bounding_box = Rectangle::new(
            Point::new(columns.start, rows.start),
            Size::new(
                (columns.end - columns.start) as u32,
                (rows.end - rows.start) as u32,
            ),
        );

        Ok(())
    }
}

fn extend_range(range: &mut Range<i32>, value: i32) {
    // MSRV: use `Range::is_empty`
    if range.start >= range.end {
        range.start = value;
        range.end = value + 1;
    } else {
        range.start = range.start.min(value);
        range.end = range.end.max(value + 1);
    }
}

impl<C> OriginDimensions for BoundingBoxDrawTarget<C> {
    fn size(&self) -> Size {
        Size::new_equal(256)
    }
}
