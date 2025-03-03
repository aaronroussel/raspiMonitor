use std::process::{Command, Stdio};
use std::{fs, io};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
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
    temp: f32,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
            self.update_temp();
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

    fn check_temps(&mut self) -> f32 {
        let output = Command::new("vcgencmd")
            .arg("measure_temp")
            .output()
            .expect("Error checking temperature");
        let stdout = String::from_utf8(output.stdout).unwrap();
        let temp = self.parse_temp_string(stdout);
        temp
    }

    fn parse_temp_string(&mut self, temp_string: String) -> f32 {
        let prefix = "temp=";
        let suffix = "'C";

        let number_str = &temp_string[prefix.len()..temp_string.len() - suffix.len() - 1];
        number_str.parse::<f32>().expect(number_str)
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
        if self.is_raspberry_pi_os() {
            self.temp = self.check_temps();
        } else {
            self.temp = 0.0;
        }
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
        let title = Line::from(" Counter App ".bold());
        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().into(),
            " Quit ".into(),
            "<Q>".blue().bold(),
            " Temperature: ".into(),
            self.temp.to_string().red().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            self.counter.to_string().yellow(),
        ])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
