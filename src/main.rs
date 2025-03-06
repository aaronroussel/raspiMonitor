use std::process::Command;
use std::time::Duration;
use std::{fs, io};

use circular_buffer::CircularBuffer;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    symbols::{self, Marker, border},
    text::{Line, Text},
    widgets::{Axis, Block, Chart, Dataset, GraphType, LegendPosition, Paragraph, Widget},
};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}

#[derive(Debug, Default)]
pub struct App {
    counter: u8,
    exit: bool,
    temp: f64,
    temp_data: TempData,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            // If we need to exit, break out of the loop
            if self.exit {
                break;
            }

            // Poll for an event for up to 200 milliseconds
            if event::poll(Duration::from_millis(200))? {
                // If there *is* an event, read it
                match event::read()? {
                    Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                        self.handle_key_event(key_event);
                    }
                    _ => {}
                };
            }

            // Update your temperature reading (and push data to the circular buffer)
            self.update_temp();

            // Re-draw the UI
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn check_temps(&mut self) -> f64 {
        let output = Command::new("vcgencmd")
            .arg("measure_temp")
            .output()
            .expect("Error checking temperature");
        let stdout = String::from_utf8(output.stdout).unwrap();
        let temp = self.parse_temp_string(stdout).clone();
        temp
    }

    fn parse_temp_string(&mut self, temp_string: String) -> f64 {
        let prefix = "temp=";
        let suffix = "'C";

        let number_str = &temp_string[prefix.len()..temp_string.len() - suffix.len() - 1];
        number_str.parse::<f64>().expect(number_str)
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left => self.decrement_counter(),
            KeyCode::Right => self.increment_counter(),
            _ => {}
        }
    }

    fn update_temp(&mut self) {
        let mut temp: f64 = 0.0;
        if self.is_raspberry_pi_os() {
            temp = self.check_temps();
            self.temp = temp;
        } else {
            self.temp = temp;
        }
        self.temp_data.add_data(temp);
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn increment_counter(&mut self) {
        self.counter += 1;
    }

    fn decrement_counter(&mut self) {
        if self.counter == 0 {
        } else {
            self.counter -= 1;
        }
    }

    fn is_raspberry_pi_os(&mut self) -> bool {
        if let Ok(contents) = fs::read_to_string("/proc/device-tree/model") {
            if contents.contains("Raspberry Pi") {
                return true;
            }
        }

        if let Ok(contents) = fs::read_to_string("/etc/os-release") {
            if contents.contains("Raspian") || contents.contains("Raspberry Pi OS") {
                return true;
            }
        }

        false
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let temp_graph_block = Block::bordered().title(Line::from(vec![
            "<".bold().green(),
            "CPU Temperature: ".into(),
            self.temp.to_string().bold().red(),
            ">".bold().green(),
        ]));

        let data = &self.temp_data.get_dataset()[..];

        let dataset = Dataset::default()
            .name("CPU TEMP")
            .data(data)
            .marker(symbols::Marker::Braille)
            .style(Style::default())
            .graph_type(GraphType::Line);

        let x_axis = Axis::default()
            .title("")
            .style(Style::default().white())
            .bounds([0.0, 100.0]);

        let y_axis = Axis::default()
            .title("CPU TEMP")
            .style(Style::default().white())
            .bounds([0.0, 100.0])
            .labels(["0.0", "25.0", "50.0", "75.0", "100.0"]);

        let temp_graph = Chart::new(vec![dataset])
            .block(temp_graph_block)
            .x_axis(x_axis)
            .y_axis(y_axis)
            .render(area, buf);
    }
}

#[derive(Debug, Clone)]
struct TempData {
    temp_buffer: CircularBuffer<100, f64>,
}

impl TempData {
    fn add_data(&mut self, data: f64) {
        self.temp_buffer.push_back(data);
    }

    fn get_dataset(&self) -> Vec<(f64, f64)> {
        self.temp_buffer
            .iter()
            .enumerate()
            .map(|(i, &val)| (i as f64, val))
            .collect()
    }
}

impl Default for TempData {
    fn default() -> Self {
        Self {
            temp_buffer: CircularBuffer::<100, f64>::new(),
        }
    }
}
