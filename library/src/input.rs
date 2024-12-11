use crossterm::event::EventStream;

use crossterm::event::{KeyCode, KeyEventKind};
use tokio_stream::StreamExt;

pub fn on_term_event(event: crossterm::event::Event, prompt: &mut String) {
    match event {
        crossterm::event::Event::Key(key) => {
            if key.kind != KeyEventKind::Press {}
            match key.code {
                KeyCode::Char(c) => {
                    *prompt = format!("{}{}", prompt, c.to_string());
                }
                KeyCode::Enter => {}
                _ => {}
            }
        }
        _ => {}
    }
}

pub async fn crossterm_wrapper_next(
    maybe_input_stream: &mut Option<EventStream>,
) -> std::option::Option<std::result::Result<crossterm::event::Event, std::io::Error>> {
    match maybe_input_stream {
        Some(input_stream) => input_stream.next().await,
        None => None,
    }
}
