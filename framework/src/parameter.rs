use embedded_graphics::prelude::*;
use std::fmt;

use crate::menu::Event;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Parameter<'a> {
    pub(crate) name: String,
    pub(crate) value: Value<'a>,
}

impl<'a> Parameter<'a> {
    pub fn new<T>(name: &str, value: &'a mut T) -> Self
    where
        Value<'a>: From<&'a mut T>,
    {
        Self {
            name: name.to_string(),
            value: Value::from(value),
        }
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Value<'a> {
    U32(&'a mut u32),
    I32(&'a mut i32),
    Point(&'a mut Point),
    Bool(&'a mut bool),
}

impl<'a> Value<'a> {
    pub(crate) fn handle_event(&mut self, event: Event) {
        match self {
            Self::U32(value) => match event {
                Event::Down | Event::Left => **value = value.saturating_sub(1),
                Event::Up | Event::Right => **value = value.saturating_add(1),
                _ => {}
            },
            Self::I32(value) => match event {
                Event::Down | Event::Left => **value = value.saturating_sub(1),
                Event::Up | Event::Right => **value = value.saturating_add(1),
                _ => {}
            },
            Self::Point(point) => match event {
                Event::Left => point.x = point.x.saturating_sub(1),
                Event::Right => point.x = point.x.saturating_add(1),
                Event::Up => point.y = point.y.saturating_sub(1),
                Event::Down => point.y = point.y.saturating_add(1),
                Event::MouseMove(p) => **point = p,
                _ => {}
            },
            Self::Bool(value) => match event {
                Event::Down | Event::Left | Event::Up | Event::Right => **value ^= true,
                _ => {}
            },
        }
    }
}

impl fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::U32(value) => value.fmt(f),
            Value::I32(value) => value.fmt(f),
            Value::Point(Point { x, y }) => write!(f, "({}, {})", x, y),
            Value::Bool(value) => value.fmt(f),
        }
    }
}

impl<'a> From<&'a mut i32> for Value<'a> {
    fn from(value: &'a mut i32) -> Self {
        Self::I32(value)
    }
}

impl<'a> From<&'a mut u32> for Value<'a> {
    fn from(value: &'a mut u32) -> Self {
        Self::U32(value)
    }
}

impl<'a> From<&'a mut Point> for Value<'a> {
    fn from(value: &'a mut Point) -> Self {
        Self::Point(value)
    }
}

impl<'a> From<&'a mut bool> for Value<'a> {
    fn from(value: &'a mut bool) -> Self {
        Self::Bool(value)
    }
}
