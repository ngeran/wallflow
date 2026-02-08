use crossterm::event::{KeyEvent, KeyEventKind};
use std::time::Duration;

pub enum Event {
    Tick,
    Key(KeyEvent),
}

pub struct EventHandler {
    /// Event sender channel
    #[allow(dead_code)]
    sender: std::sync::mpsc::Sender<Event>,
    /// Event receiver channel
    receiver: std::sync::mpsc::Receiver<Event>,
    /// Event handler thread handle
    #[allow(dead_code)]
    handle: std::thread::JoinHandle<()>,
}

impl EventHandler {
    /// Create a new event handler
    pub fn new(tick_rate_ms: u64) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        let sender_clone = sender.clone();

        let handle = std::thread::spawn(move || {
            let mut last_tick = std::time::Instant::now();
            let tick_duration = Duration::from_millis(tick_rate_ms);

            loop {
                let timeout = tick_duration
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if crossterm::event::poll(timeout).unwrap_or(false) {
                    if let crossterm::event::Event::Key(key) = crossterm::event::read().unwrap() {
                        // Only send key press events, not release events
                        if key.kind == KeyEventKind::Press {
                            sender_clone.send(Event::Key(key)).unwrap();
                        }
                    }
                }

                if last_tick.elapsed() >= tick_duration {
                    sender_clone.send(Event::Tick).unwrap();
                    last_tick = std::time::Instant::now();
                }
            }
        });

        Self {
            sender,
            receiver,
            handle,
        }
    }

    /// Receive the next event
    pub fn next(&self) -> Result<Event, std::sync::mpsc::RecvError> {
        self.receiver.recv()
    }
}
