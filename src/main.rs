pub mod engine;
mod lexer;
mod parser;
mod tests;
mod utils;

use std::fs::write;
use std::time::Instant;
use std::{fs::read_to_string, io};

use crate::engine::Engine;
use crate::parser::error::ParserError;

#[derive(Debug)]
enum DLErr {
    LexerError(lexer::LexerError),
    IOError(std::io::Error),
    ParserError(ParserError),
    RuntimeError(engine::RuntimeError),
}

impl From<lexer::LexerError> for DLErr {
    fn from(e: lexer::LexerError) -> Self {
        Self::LexerError(e)
    }
}

impl From<ParserError> for DLErr {
    fn from(e: ParserError) -> Self {
        Self::ParserError(e)
    }
}

impl From<engine::RuntimeError> for DLErr {
    fn from(e: engine::RuntimeError) -> Self {
        Self::RuntimeError(e)
    }
}

impl From<std::io::Error> for DLErr {
    fn from(e: std::io::Error) -> Self {
        Self::IOError(e)
    }
}
const AUTO_RUN_SAMPLES: usize = 30;

fn console_loop() -> Result<(), DLErr> {
    let mut engine = Engine::new();

    engine.set_recursion_limit(10);

    let stdin = io::stdin();

    loop {
        let mut buffer = String::new();

        print!("\n>");
        io::Write::flush(&mut io::stdout())
            .ok()
            .expect("Could not flush stdout");
        stdin.read_line(&mut buffer)?;

        if buffer.chars().nth(0).unwrap_or_else(|| unreachable!()) == '/' {
            if buffer.starts_with("/exit") {
                break;
            }
            if buffer.starts_with("/import") {
                let file_path: String = buffer
                    .chars()
                    .into_iter()
                    .skip_while(|c| c != &' ')
                    .skip(1)
                    .collect();
                match read_to_string(file_path.trim()) {
                    Ok(commands) => println!("{}", engine.input(commands)),
                    Err(err) => println!(
                        "the file couldnt be read ({}), reason: {err}",
                        file_path.trim()
                    ),
                }
            }
            if buffer.starts_with("/export") {
                let file_path: String = buffer
                    .chars()
                    .into_iter()
                    .skip_while(|c| c != &' ')
                    .skip(1)
                    .collect();
                match write(file_path.trim(), format!("{engine}")) {
                    Ok(_) => println!("ok"),
                    Err(err) => println!("export failed due to: {err}"),
                }
            }

            if buffer.starts_with("/set_recursion_limit") {
                let arg: String = buffer
                    .chars()
                    .into_iter()
                    .skip_while(|c| c != &' ')
                    .skip(1)
                    .collect();
                match arg.parse::<usize>() {
                    Ok(num) => engine.set_recursion_limit(num),
                    Err(err) => println!("error parsing argument: {err:?}"),
                }
            }
        } else {
            println!("{}", engine.input(buffer));
        }
    }

    Ok(())
}

fn main() -> Result<(), DLErr> {
    if AUTO_RUN_SAMPLES > 0 {
        let mut times = vec![];

        for i in 0..AUTO_RUN_SAMPLES {
            let now = Instant::now();

            let mut engine = Engine::new();

            engine.set_recursion_limit(5);

            engine.input(read_to_string("debug_input.dl")?);

            let elapsed_time = now.elapsed();
            println!(
                "{}%: {}s",
                100. * (i as f64) / (AUTO_RUN_SAMPLES as f64),
                elapsed_time.as_secs_f64()
            );
            times.push(elapsed_time.as_micros());
        }

        times.sort();
        // let min = times[0];
        // let max = times[times.len() - 1];
        let mut sum = 0;
        for t in &times {
            sum += t;
        }

        println!(
            "{:?} => {}",
            times,
            (sum as f64) / (AUTO_RUN_SAMPLES as f64)
        );
    } else {
        console_loop()?;
    }

    Ok(())
}
