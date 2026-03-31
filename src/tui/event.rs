use crate::tui::action::Action;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

pub struct EventHandler {
    receiver: mpsc::Receiver<Action>,
    sender: mpsc::Sender<Action>,
    tick_rate: u64,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let (sender, receiver) = mpsc::channel();
        
        let tx = sender.clone();
        thread::spawn(move || {
            let mut last_tick = Instant::now();
            let tick_duration = Duration::from_millis(tick_rate);
            
            loop {
                let timeout = tick_duration
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));
                
                if crossterm::event::poll(timeout).expect("Failed to poll crossterm events") {
                    if let Event::Key(key) = event::read().expect("Failed to read crossterm event") {
                        if tx.send(Self::map_key_action(key)).is_err() {
                            break;
                        }
                    }
                }
                
                if last_tick.elapsed() >= tick_duration {
                    if tx.send(Action::Tick).is_err() {
                        break;
                    }
                    last_tick = Instant::now();
                }
            }
        });

        Self {
            receiver,
            sender,
            tick_rate,
        }
    }

    pub fn sender(&self) -> mpsc::Sender<Action> {
        self.sender.clone()
    }

    pub fn next(&self) -> anyhow::Result<Action> {
        Ok(self.receiver.recv()?)
    }
    
    fn map_key_action(key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
            KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => Action::NextTab,
            KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => Action::PrevTab,
            KeyCode::Down | KeyCode::Char('j') => Action::ScrollDown,
            KeyCode::Up | KeyCode::Char('k') => Action::ScrollUp,
            KeyCode::Enter => Action::Select,
            KeyCode::Char('r') => Action::Refresh,
            _ => Action::Tick, // Ignore unimplemented keys
        }
    }
}
