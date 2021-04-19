use embedded_graphics::{
    mono_font::{latin1::FONT_6X10, MonoTextStyle, MonoTextStyleBuilder},
    prelude::*,
    text::Text,
};
use embedded_graphics_simulator::{SimulatorEvent, Window};
use sdl2::{keyboard::Keycode, mouse::MouseButton};

use crate::Parameter;

pub struct Menu {
    selected: usize,
    active: bool,
    mouse_button_down: bool,
}

impl Menu {
    pub(crate) fn new() -> Self {
        Self {
            selected: 0,
            active: false,
            mouse_button_down: false,
        }
    }

    pub(crate) fn draw_menu<T>(
        &self,
        parameters: &[Parameter],
        target: &mut T,
        color: T::Color,
    ) -> Result<(), T::Error>
    where
        T: DrawTarget,
    {
        let max_name_width = parameters
            .iter()
            .map(|parameter| parameter.name.len())
            .max()
            .unwrap_or(0);

        let name_delta = Point::new(6, 0);
        let value_delta = name_delta + Point::new((max_name_width as i32 + 1) * 6, 0);

        let style = MonoTextStyle::new(&FONT_6X10, color);
        let style_inverted = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .background_color(color)
            .build();

        let mut position = Point::new(2, 8);

        for (index, parameter) in parameters.iter().enumerate() {
            let item_style = if index == self.selected && self.active {
                style_inverted
            } else {
                style
            };

            if index == self.selected {
                Text::new(">", position, style).draw(target)?;
            }

            Text::new(&parameter.name, position + name_delta, item_style).draw(target)?;
            Text::new(&parameter.value.to_string(), position + value_delta, style).draw(target)?;

            position.y += 10;
        }

        Ok(())
    }

    pub(crate) fn handle_events(
        &mut self,
        parameters: &mut [Parameter],
        window: &mut Window,
    ) -> bool {
        for event in window.events() {
            let event = match event {
                SimulatorEvent::Quit => return true,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Up => Event::Up,
                    Keycode::Down => Event::Down,
                    Keycode::Left => Event::Left,
                    Keycode::Right => Event::Right,
                    Keycode::Space | Keycode::Return => Event::Activate,
                    _ => continue,
                },
                SimulatorEvent::MouseButtonDown { mouse_btn, point }
                    if mouse_btn == MouseButton::Left =>
                {
                    self.mouse_button_down = true;
                    Event::MouseMove(point)
                }
                SimulatorEvent::MouseButtonDown { mouse_btn, .. }
                    if mouse_btn == MouseButton::Middle =>
                {
                    Event::Activate
                }
                SimulatorEvent::MouseMove { point } if self.mouse_button_down => {
                    Event::MouseMove(point)
                }
                SimulatorEvent::MouseButtonUp { .. } => {
                    self.mouse_button_down = false;
                    continue;
                }
                _ => continue,
            };

            match event {
                Event::Up if !self.active => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    } else {
                        self.selected = parameters.len() - 1;
                    }
                }
                Event::Down if !self.active => {
                    self.selected += 1;
                    if self.selected >= parameters.len() {
                        self.selected = 0;
                    }
                }
                Event::Activate => self.active ^= true,
                _ => parameters[self.selected].value.handle_event(event),
            }
        }

        false
    }
}

pub(crate) enum Event {
    Up,
    Down,
    Left,
    Right,
    Activate,
    MouseMove(Point),
}
