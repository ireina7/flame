use std::io::{self, Write};

use anyhow::anyhow;
use clap::Parser;
use colored::*;

use crate::{
    review::{logic, Quality, Retriever},
    words::{App, Item, Word},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub path: String,

    #[arg(short, long)]
    pub count_as_a_day: bool,
}

const INFO: &str = "[INFO]";
const MARK: &str = "[MARK]";
const INPUT: &str = ">";

pub fn run() -> anyhow::Result<()> {
    let args = Args::parse();
    println!("{}\n", logo());

    let mut app = App::from(&args.path, false)?;
    let ids = app.retrieve()?;
    println!(
        "{} Totally {} items to review today.",
        INFO.blue(),
        ids.len()
    );
    let mut cnt = 0;

    app.count_as_a_day = args.count_as_a_day;
    let quit = logic::<_, anyhow::Error>(&mut app, |item| {
        let ratio = cnt as f64 / ids.len() as f64;
        println!(
            "\n{}{}",
            (0..(100.0 * ratio).ceil() as usize)
                .map(|_| "#")
                .collect::<Vec<&str>>()
                .join(""),
            (0..(100 - ((100.0 * ratio).ceil() as usize)))
                .map(|_| "-")
                .collect::<Vec<&str>>()
                .join("")
        );
        let ans = handle_item(item);
        cnt += 1;
        ans
    })?;
    if quit {
        println!("\n{} Saving progress...", INFO.blue());
        app.save(&args.path)?;

        println!("{} Bye bye.", INFO.blue());
        return Ok(());
    }

    println!("\n{} Saving progress...", INFO.blue());
    app.save(&args.path)?;

    println!("{} Bye bye.", INFO.blue());
    Ok(())
}

fn handle_item(item: &Item<Word>) -> anyhow::Result<Option<Quality>> {
    println!("[{}]", item.payload.word.bold().bright_green().italic());
    termimad::print_inline(item.payload.detail()?.as_str());
    println!("\n");
    println!(
        "{} blackout(b) | incorrect(i) | correct but hard(h) | correct(c) | perfect(f):",
        MARK.bright_purple()
    );
    print!("{} ", INPUT.bright_yellow());
    io::stdout().flush().unwrap();
    read_quality()
}

fn read_quality() -> anyhow::Result<Option<Quality>> {
    let mut ans = String::new();
    io::stdin()
        .read_line(&mut ans)
        .expect("Failed to read line");
    let q = match ans.as_str().trim() {
        "b" => 0,
        "i" => 1,
        "h" => 3,
        "c" => 4,
        "f" => 5,
        "q" => return Ok(None),
        x => return Err(anyhow!("unknown mark: {}", x)),
    };

    Quality::from(q)
        .ok_or(anyhow!("failed to get quality, found u8: {q:?}"))
        .map(Some)
}

const fn logo() -> &'static str {
    r#"  _____.__                         
_/ ____\  | _____    _____   ____  
\   __\|  | \__  \  /     \_/ __ \ 
 |  |  |  |__/ __ \|  Y Y  \  ___/ 
 |__|  |____(____  /__|_|  /\___  >
                 \/      \/     \/ "#
}
