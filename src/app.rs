use crossterm::event::{self, Event, KeyCode, KeyEvent};
use parversion::prelude::{ExecutionContext, ProgressEvent};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    prelude::*,
    style::Stylize,
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Widget},
};
use std::io;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use crate::constants::HOLD_TO_REGENERATE_SECONDS;
use crate::constants::colors::{
    STATUS_BAR_INTERACTION_COLOR, STATUS_BAR_NAVIGATION_COLOR, STATUS_BAR_NAVIGATION_INPUT_COLOR,
};
use crate::content::ContentPayload;
use crate::content::digest::Digest;
use crate::context::Context;
use crate::loading_context::{LoadingContext, StageMessage};
use crate::prelude::*;
use crate::ui::UI;

pub struct App {
    context: Context,
    ui: UI,
    exit: bool,
    tx: mpsc::UnboundedSender<ContentPayload>,
    rx: mpsc::UnboundedReceiver<ContentPayload>,
    double_tap_window: Duration,
    held_key: Option<KeyCode>,
    hold_start: Option<Instant>,
    last_press: Option<Instant>,
    double_tap_pending: bool,
    regen_triggered: bool,
    loading_context: Option<Arc<RwLock<LoadingContext>>>,
}

impl App {
    pub fn new(context: Context) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            context,
            ui: UI::new(),
            exit: false,
            tx,
            rx,
            double_tap_window: Duration::from_millis(350),
            held_key: None,
            hold_start: None,
            last_press: None,
            double_tap_pending: false,
            regen_triggered: false,
            loading_context: None,
        }
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().await?;

            if let Ok(content) = self.rx.try_recv() {
                self.ui.run(content);
                self.loading_context = None;
                self.context.set_mode(Mode::Interaction);
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    async fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                self.handle_key_event(key_event).await;
            }
        }

        self.process_timers();
        Ok(())
    }

    fn handle_navigation_key_event(&mut self, key_event: KeyEvent) -> Option<Action> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('r') => self.on_special_key_press(KeyCode::Char('r')),
            KeyCode::Enter => self.navigate(false),
            _ => {
                self.clear_key_state();
            }
        }

        None
    }

    fn handle_navigation_input_key_event(&mut self, key_event: KeyEvent) -> Option<Action> {
        match key_event.code {
            KeyCode::Char(c) => {
                self.context.append_char(c);
            }
            KeyCode::Backspace => {
                self.context.remove_last_char();
            }
            KeyCode::Enter => {
                self.navigate(false);
            }
            _ => {}
        }

        None
    }

    fn handle_universal_key_event(&mut self, key_event: KeyEvent) -> Option<Action> {
        match key_event.code {
            KeyCode::Esc => {
                self.context.set_mode(Mode::Navigation);
            }
            KeyCode::Char('/') => {
                self.context.set_mode(Mode::NavigationInput);
            }
            _ => {}
        }

        None
    }

    async fn handle_key_event(&mut self, key_event: KeyEvent) {
        self.handle_universal_key_event(key_event);

        let action = {
            match self.context.get_mode().clone() {
                Mode::Navigation => self.handle_navigation_key_event(key_event),
                Mode::Interaction => self.ui.handle_key_event(key_event),
                Mode::NavigationInput => self.handle_navigation_input_key_event(key_event),
            }
        };

        if let Some(action) = action {
            self.handle_action(action);
        }
    }

    fn on_special_key_press(&mut self, code: KeyCode) {
        let now = Instant::now();

        if self.held_key != Some(code) {
            self.clear_key_state();
            self.held_key = Some(code);
        }

        if self.hold_start.is_none() {
            self.hold_start = Some(now);
            self.regen_triggered = false;
        }

        if self.double_tap_pending {
            if let Some(last) = self.last_press {
                if now.duration_since(last) > Duration::from_millis(100)
                    && now.duration_since(last) <= self.double_tap_window
                {
                    self.refresh();
                    self.clear_key_state();
                    return;
                }
            }
        }

        self.double_tap_pending = true;
        self.last_press = Some(now);
    }

    fn process_timers(&mut self) {
        if let Some(start) = self.hold_start {
            if !self.regen_triggered
                && start.elapsed() >= Duration::from_secs(HOLD_TO_REGENERATE_SECONDS)
            {
                self.regenerate();
                self.regen_triggered = true;
                self.double_tap_pending = false;
            }
        }

        if self.double_tap_pending {
            if let Some(last) = self.last_press {
                if last.elapsed() > self.double_tap_window {
                    self.double_tap_pending = false;
                    self.last_press = None;
                }
            }
        }
    }

    fn clear_key_state(&mut self) {
        self.held_key = None;
        self.hold_start = None;
        self.last_press = None;
        self.double_tap_pending = false;
        self.regen_triggered = false;
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn regenerate(&mut self) {
        self.navigate(true);
    }

    fn refresh(&mut self) {
        self.navigate(false);
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            //Action::Open(_url) => unimplemented!(),
            Action::OpenUsingRenderingEngine(url) => {
                self.context.open_using_system(url);
            }
        }
    }

    fn navigate(&mut self, regenerate: bool) {
        let loading_context = Arc::new(RwLock::new(LoadingContext::new()));
        self.loading_context = Some(Arc::clone(&loading_context));

        let (tx, mut rx) = mpsc::unbounded_channel();
        let execution_context = ExecutionContext::with_progress(tx);

        let loading_context_clone = Arc::clone(&loading_context);
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                let mut loading_context = loading_context_clone.write().unwrap();

                match event {
                    ProgressEvent::StageStart(stage) => {
                        loading_context
                            .stage_messages
                            .push((stage.to_string(), Vec::new()));
                    }
                    ProgressEvent::StageDone(stage) => {
                        if let Some((_, messages)) = loading_context
                            .stage_messages
                            .iter_mut()
                            .find(|(s, _)| s == &stage)
                        {
                            messages.push(StageMessage {
                                message: format!("{} complete", stage),
                                tokens: 0,
                            });
                        }
                    }
                    ProgressEvent::Event {
                        stage,
                        event_name,
                        tokens,
                    } => {
                        if let Some((_, messages)) = loading_context
                            .stage_messages
                            .iter_mut()
                            .find(|(s, _)| s == &stage)
                        {
                            messages.push(StageMessage {
                                message: event_name.to_string(),
                                tokens: tokens,
                            });
                        }

                        *loading_context
                            .stage_tokens
                            .entry(stage.to_string())
                            .or_insert(0) += tokens;
                        loading_context.global_tokens += tokens;
                    }
                }
            }
        });

        let context_clone = self.context.clone();
        let execution_context_clone = execution_context.clone();
        let tx_clone = self.tx.clone();

        tokio::spawn(async move {
            let content_payload: ContentPayload = context_clone
                .open(execution_context_clone, regenerate)
                .await
                .expect("Could not open URL");

            tx_clone.send(content_payload).unwrap();
        });
    }
}

impl App {
    fn render_header(&mut self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" pori ".bold());
        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::ROUNDED);

        let url = self.context.url_to_string();

        let search_text = if let Mode::NavigationInput = self.context.get_mode() {
            Text::from(vec![Line::from(vec![
                Span::raw("Navigate: ").white(),
                Span::raw(url),
            ])])
        } else {
            Text::from(vec![Line::from(vec![Span::raw(url)])])
        };

        Paragraph::new(search_text)
            .centered()
            .block(block)
            .render(area, buf);
    }

    fn render_body(&mut self, area: Rect, buf: &mut Buffer) {
        if let Some(loading_context) = &self.loading_context {
            let mut lines: Vec<Line> = vec![Line::from(Span::styled(
                "Loading page",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Magenta),
            ))];

            let guard = loading_context.read().unwrap();

            for (stage_name, messages) in &guard.stage_messages {
                // Stage heading
                lines.push(Line::from(Span::styled(
                    stage_name.clone(),
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Yellow),
                )));

                // Stage messages
                struct GroupedMessage<'a> {
                    message: &'a str,
                    count: usize,
                    tokens: u64,
                }

                let mut grouped_messages: Vec<GroupedMessage> = Vec::new();

                for message in messages {
                    if let Some(last) = grouped_messages.last_mut() {
                        if last.message == message.message {
                            last.count += 1;
                            last.tokens += message.tokens;
                            continue;
                        }
                    }

                    grouped_messages.push(GroupedMessage {
                        message: &message.message,
                        count: 1,
                        tokens: message.tokens,
                    });
                }

                for group in grouped_messages {
                    let mut text = format!("    - {}", group.message);

                    if group.count > 1 {
                        text.push_str(&format!(" ({})", group.count));
                    }

                    text.push_str(&format!(" {} tokens", group.tokens));

                    lines.push(Line::from(text));
                }

                // Stage tokens
                if let Some(tokens) = &guard.stage_tokens.get(stage_name) {
                    lines.push(Line::from(Span::styled(
                        format!("   Stage tokens: {}", tokens),
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::ITALIC),
                    )));
                }

                lines.push(Line::default());
            }

            // Total tokens
            lines.push(Line::from(Span::styled(
                format!("Total tokens: {}", &guard.global_tokens),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Green),
            )));

            Paragraph::new(lines).render(area, buf);
        } else {
            self.ui.render(area, buf);
        }
    }

    fn render_status_bar(&mut self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let mode = self.context.get_mode();

        let style = {
            match mode {
                Mode::Navigation => Style::default().fg(STATUS_BAR_NAVIGATION_COLOR),
                Mode::Interaction => Style::default().fg(STATUS_BAR_INTERACTION_COLOR),
                Mode::NavigationInput => Style::default().fg(STATUS_BAR_NAVIGATION_INPUT_COLOR),
            }
        };

        Paragraph::new(mode.as_str())
            .style(style)
            .render(layout[0], buf);

        Paragraph::new(format!("v{}", env!("CARGO_PKG_VERSION")))
            .style(Style::default())
            .alignment(Alignment::Right)
            .render(layout[1], buf);
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(area);

        self.render_header(layout[0], buf);
        self.render_body(layout[1], buf);
        self.render_status_bar(layout[2], buf);
    }
}
