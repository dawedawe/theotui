pub(crate) mod model;
pub(crate) mod update;
pub(crate) mod view;

use std::{env, fs};

use model::Model;
use update::{handle_event, update};
use view::view;

fn main() -> color_eyre::Result<()> {
    let mut model = Model::default();
    let args: Vec<_> = env::args().collect();
    read_inputs_from_args(args, &mut model);

    color_eyre::install()?;
    let mut terminal = ratatui::init();

    while model.running {
        terminal.draw(|f| view(&mut model, f))?;
        if let Some(msg) = handle_event(&mut model)? {
            update(&mut model, msg)
        }
    }

    ratatui::restore();
    color_eyre::Result::Ok(())
}

fn usage() -> ! {
    eprintln!("Usage: theotui [--dfa file]");
    std::process::exit(1)
}

fn read_inputs_from_args(args: Vec<String>, model: &mut Model) {
    if args.len() == 1 {
    } else if args.len() == 3 && args[1] == "--dfa" {
        match fs::read_to_string(&args[2]) {
            Result::Ok(s) => {
                model.dfa_state.definition_textarea.select_all();
                model.dfa_state.definition_textarea.cut();
                model.dfa_state.definition_textarea.insert_str(s);
            }
            Err(e) => {
                eprintln!("{e}");
                usage();
            }
        }
    } else {
        usage();
    }
}
