use std::{cell::RefCell, io, rc::Rc};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, MouseEvent, MouseEventKind};
use ratatui::{
    layout::{Layout, Constraint, Direction, Rect},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame, DefaultTerminal,
};
use ratatui::prelude::{Color, Style};
use ratatui::style::Modifier;
use ratatui::text::Span;
use crate::cpu::instruction_set::DebugInstruction;
use crate::mmu::MemoryRegion;
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
    pub breakpoints: Vec<u16>,
    pub last_hit_breakpoint: Option<u16>,
    scroll: i16,
    backward_instructions_count: usize,
}

impl App {
    pub fn new(rainier: Rc<RefCell<Rainier>>) -> Self {
        let breakpoints: Vec<u16> = vec![0xC6D6];

        Self {
            rainier,
            requested_action: None,
            exit: false,
            current_instruction_set: Vec::new(),
            current_instruction_id: 0,
            breakpoints,
            last_hit_breakpoint: None,
            scroll: 0,
            backward_instructions_count: 5
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        {
            let rainier = self.rainier.borrow();
            let cpu = rainier.cpu.borrow();

            let instructions = cpu.dump_instructions(cpu.registers.pc() as usize);
            self.current_instruction_set = instructions;

            self.current_instruction_id = self.current_instruction_set.iter().position(|r| r.address == cpu.registers.pc() as usize).unwrap()
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
            Span::styled("  Trace", Style::default()),
            Span::styled( "<F1>", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("  Step Over", Style::default()),
            Span::styled( "<F2>", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("  Run", Style::default()),
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
        self.draw_disassembly(frame, chunks[0]);

        // Right panel: registers
        self.draw_registers(frame, chunks[1]);
    }

    fn draw_registers(&self, frame: &mut Frame, area: Rect) {
        let rainier = self.rainier.borrow();
        let cpu = rainier.cpu.borrow();

        let lines: Vec<Line> = vec![
            Line::from(format!("AF: {:04X}    Z: {}", cpu.registers.af(), if cpu.registers.zero_flag() { "✓" } else { "X" })),
            Line::from(format!("BC: {:04X}    N: {}", cpu.registers.bc(), if cpu.registers.subtraction_flag() { "✓" } else { "X" })),
            Line::from(format!("DE: {:04X}    H: {}", cpu.registers.de(), if cpu.registers.half_carry_flag() { "✓" } else { "X" })),
            Line::from(format!("HL: {:04X}    C: {}", cpu.registers.hl(), if cpu.registers.carry_flag() { "✓" } else { "X" })),
            Line::from(format!("SP: {:04X}", cpu.registers.sp())),
            Line::from(format!("PC: {:04X}", cpu.registers.pc())),
            Line::from(format!("0xC000: {:02X}", rainier.mmu.borrow().read_byte(0xC000).unwrap()))];

        let block = Block::default().title("Registers").borders(Borders::ALL);
        let registers = Paragraph::new(lines).block(block);

        frame.render_widget(registers, area);
    }

    fn draw_disassembly(&self, frame: &mut Frame, area: Rect) {
        let rainier = self.rainier.borrow();

        let starting_point = self.current_instruction_id - self.backward_instructions_count - self.scroll as usize;
        let lines = &self.current_instruction_set[starting_point..starting_point+50];

        let lines = lines
            .iter()
            .enumerate()
            .map(|(i, instruction)| {
                let breakpoint = if self.breakpoints.contains(&(instruction.address as u16)) { "🟠" } else { "  " };
                let prefix = if i == self.backward_instructions_count + self.scroll as usize { "▶" } else { " " };
                let memory_region = MemoryRegion::from_address(instruction.address).unwrap().as_str();
                let first_operand = instruction.first_operand.map_or(String::from("  "), |operand| format!("{:02X}", operand));
                let second_operand = instruction.second_operand.map_or(String::from("  "), |operand| format!("{:02X}", operand));

                Line::from(format!("{} {} {}:{:04X} {:02X} {} {}        {}",
                    breakpoint,
                    prefix,
                    memory_region,
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
            KeyCode::F(3) => {
                self.requested_action = Some(Action::Run);
                self.scroll = 0;
            }
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