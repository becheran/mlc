use std::error::Error;
use std::{fs, io};
use clap::{Arg, App};

#[macro_use]
extern crate clap;

arg_enum! {
    #[derive(Debug)]
    pub enum Difficulty {
        Easy,
        Normal,
        Hard
    }
}

pub struct Config {
    pub difficulty: Difficulty, // TODO
}

impl Config {
    pub fn new() -> Config {
        let matches = App::new("cliq")
            .version("0.1.0")
            .author("Armin Becher <becherarmin@gmail.com>")
            .about("Command Line IQ (CLIQ) trainer. Train your number sequence solving skills.")
            .arg(Arg::with_name("difficulty")
                .short("d")
                .long("difficulty")
                .takes_value(true)
                .help("The difficulty level of the puzzles")
                .possible_values(&Difficulty::variants()))
            .get_matches();

        let difficulty = value_t!(matches.value_of("difficulty"),Difficulty).unwrap_or(Difficulty::Normal);
        println!("The difficulty level is: {}", difficulty);
        Config { difficulty }
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    println!("++++++++++++++++++++++++++++++++++");
    println!("++++++++++ link-checker ++++++++++");
    println!("++++++++++++++++++++++++++++++++++");

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn dummy_test() {
        assert_eq!(1, 1);
    }
}