use std::error::Error;
#[macro_use]
extern crate clap;
use clap::App;

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
        let yaml = load_yaml!("cli.yml");
        let matches = App::from_yaml(yaml).get_matches();
        let difficulty = value_t!(matches.value_of("difficulty"),Difficulty).unwrap_or(Difficulty::Normal);
        println!("The difficulty level is: {}", difficulty);
        Config { difficulty }
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    println!("++++++++++++++++++++++++++++++++++");
    println!("++++++++++ linkchecker ++++++++++");
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