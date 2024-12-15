use std::io::{stdin, stdout, IsTerminal, Write};

use crossterm::event::EventStream;

use crossterm::event::{KeyCode, KeyEventKind};
use log::trace;
use tokio_stream::StreamExt;

pub fn on_term_event(event: crossterm::event::Event, prompt: &mut String) -> bool {
    println!("--------");
    log::error!("===========");
    match event {
        crossterm::event::Event::Key(key) => {
            if stdin().is_terminal() && key.kind != KeyEventKind::Press {
                log::error!("exit");
                return false;
            }
            match key.code {
                KeyCode::Char(c) => {
                    *prompt = format!("{}{}", prompt, c.to_string());
                    print!("{}", c);
                    stdout().flush().unwrap();
                    if !stdin().is_terminal() {
                        if prompt == "stop" {
                            println!();
                            return true;
                        }
                    }
                }
                KeyCode::Enter => {
                    if prompt.is_empty() {
                        return false;
                    }
                    let old_prompt = prompt.clone();
                    prompt.clear();
                    println!();
                    if old_prompt == "stop" {
                        return true;
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    false
}

pub async fn crossterm_wrapper_next(
    maybe_input_stream: &mut Option<EventStream>,
) -> std::option::Option<std::result::Result<crossterm::event::Event, std::io::Error>> {
    match maybe_input_stream {
        Some(input_stream) => {
            println!("ojfsofdfssd");
            trace!("Some input");
            input_stream.next().await
        }
        None => {
            println!("PLOOOOP");
            trace!("No input");
            None
        }
    }
}
