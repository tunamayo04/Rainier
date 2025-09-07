use std::{cell::RefCell, io, rc::Rc};
use std::io::stdout;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, MouseEvent, MouseEventKind};
use ratatui::{
    layout::{Layout, Constraint, Direction, Rect},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame, DefaultTerminal,
};
use ratatui::crossterm::event::EnableMouseCapture;
use ratatui::crossterm::execute;
use ratatui::prelude::{Color, Style};
use ratatui::style::Modifier;
use ratatui::text::Span;
use crate::cpu::instruction_set::DebugInstruction;
use crate::Rainier;

#[derive(Eq, PartialEq)]
pub enum Action {
    Trace,
    StepOver,
    Run,
}

pub struct App {
    rainier: Rc<RefCell<Rainier>>,
    pub requested_action: Option<Action>,
    pub exit: bool,
    current_instruction_set: Vec<DebugInstruction>,
    current_instruction_id: usize,
    scroll: i16,
    backward_instructions_count: usize,
}

impl App {
    pub fn new(rainier: Rc<RefCell<Rainier>>) -> Self {
        Self {
            rainier,
            requested_action: None,
            exit: false,
            current_instruction_set: Vec::new(),
            current_instruction_id: 0,
            scroll: 0,
            backward_instructions_count: 5
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        execute!(stdout(), EnableMouseCapture)?;

        {
            let rainier = self.rainier.borrow();

            if self.current_instruction_set.is_empty() {
                let rainier = self.rainier.borrow();
                let instructions = rainier.cpu.dump_instructions(rainier.cpu.registers.pc() as usize);
                self.current_instruction_set = instructions;
            }

            self.current_instruction_id = self.current_instruction_set.iter().position(|r| r.address == rainier.cpu.registers.pc() as usize).unwrap()
        }

        terminal.draw(|frame| self.draw(frame))?;
        self.handle_events()?;
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let outer_area = frame.area();

        // Outer block
        let title = Line::from("Rainier debugger");
        let instructions = Line::from(vec![
            Span::styled(" Quit", Style::default()),
            Span::styled("<Q>", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(" Trace", Style::default()),
            Span::styled( "<F1>", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(" Step Over", Style::default()),
            Span::styled( "<F2>", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(" Run", Style::default()),
            Span::styled( "<F3>", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]);
        let outer_block = Block::default()
            .title(title.centered())
            .title_bottom(instructions)
            .borders(Borders::ALL);

        frame.render_widget(outer_block.clone(), outer_area);

        let inner_area = outer_block.inner(outer_area);

        // Use layout instead of manual positioning
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(inner_area);

        // Left panel: maybe disassembly/logs later
        self.draw_placeholder(frame, chunks[0]);

        // Right panel: registers
        self.draw_registers(frame, chunks[1]);
    }

    fn draw_registers(&self, frame: &mut Frame, area: Rect) {
        let rainier = self.rainier.borrow();
        let lines: Vec<Line> = vec![
            Line::from(format!("AF: {:04X}    Z: {}", rainier.cpu.registers.af(), if rainier.cpu.registers.zero_flag() { "✓" } else { "X" })),
            Line::from(format!("BC: {:04X}    N: {}", rainier.cpu.registers.bc(), if rainier.cpu.registers.subtraction_flag() { "✓" } else { "X" })),
            Line::from(format!("DE: {:04X}    H: {}", rainier.cpu.registers.de(), if rainier.cpu.registers.half_carry_flag() { "✓" } else { "X" })),
            Line::from(format!("HL: {:04X}    C: {}", rainier.cpu.registers.hl(), if rainier.cpu.registers.carry_flag() { "✓" } else { "X" })),
            Line::from(format!("SP: {:04X}", rainier.cpu.registers.sp())),
            Line::from(format!("PC: {:04X}", rainier.cpu.registers.pc()))];

        let block = Block::default().title("Registers").borders(Borders::ALL);
        let registers = Paragraph::new(lines).block(block);

        frame.render_widget(registers, area);
    }

    fn draw_placeholder(&self, frame: &mut Frame, area: Rect) {
        let rainier = self.rainier.borrow();

        let lines = self.current_instruction_set
            .iter()
            .skip(self.current_instruction_id - self.backward_instructions_count - self.scroll as usize)
            .enumerate()
            .map(|(i, instruction)| {
                let prefix = if i == self.backward_instructions_count + self.scroll as usize { "▶" } else { " " };
                let first_operand = instruction.first_operand.map_or(String::from("  "), |operand| format!("{:02X}", operand));
                let second_operand = instruction.second_operand.map_or(String::from("  "), |operand| format!("{:02X}", operand));

                Line::from(format!(" {} ROM0:{:04X} {:02X} {} {}        {}",
                    prefix,
                    instruction.address,
                    instruction.opcode,
                    first_operand,
                    second_operand,
                    instruction.name)
                )})
            .collect::<Vec<Line>>();

        let block = Block::default().title("Disassembly").borders(Borders::ALL);
        let ops = Paragraph::new(lines).block(block);

        frame.render_widget(ops, area);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            },
            Event::Mouse(mouse_event) => {
                self.handle_mouse_event(mouse_event);
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::F(1) => {
                self.requested_action = Some(Action::Trace);
                self.scroll = 0;
            },
            _ => {}
        }
    }

    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) {
        match mouse_event.kind {
            // TODO: Bounds check
            MouseEventKind::ScrollDown => {
                self.scroll -= 1;
            },
            MouseEventKind::ScrollUp => {
                //if (self.scroll as usize) < self.current_instruction_id - self.backward_instructions_count {
                 //   self.scroll += 1;
                //}
                self.scroll += 1;
            }
            _ => {}
        }
    }
}