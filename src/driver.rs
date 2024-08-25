use std::{
    fs,
    io::{self, Read, Write},
    sync::OnceLock,
};

use anyhow::anyhow;
use clap::{Parser, Subcommand};
use colored::*;
use termimad::MadSkin;

use crate::{
    review::{logic, Quality, Retriever},
    words::{App, Item, Word},
};

static SKIN: OnceLock<MadSkin> = OnceLock::new();

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path of index json
    #[arg(short, long)]
    pub path: String,

    /// Whether count as a day
    #[arg(short, long)]
    pub count_as_a_day: bool,

    /// Repeat
    #[arg(short, long)]
    pub repeat: bool,

    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Launch
    Up,

    /// Add word
    Add { name: String, path: String },

    /// Delete word
    Delete { name: String },

    /// Show word
    Show { name: String },

    /// Clear words
    Clear,
}

pub const INFO: &str = "[INFO]";
pub const MARK: &str = "[MARK]";
pub const INPUT: &str = ">";

pub fn run(args: Args) -> anyhow::Result<()> {
    let mut skin = MadSkin::default();
    skin.italic
        .set_fg(termimad::crossterm::style::Color::Yellow);
    skin.bold.set_fg(termimad::crossterm::style::Color::Green);
    // skin.print_inline("*Hey* **World!** Here's `some(code)`");
    // skin.paragraph
    //     .set_fgbg(termimad::crossterm::style::Color::Magenta, rgb(30, 30, 40));

    // println!(
    //     "\nand now {}\n",
    //     skin.inline("a little *too much* **style!** (and `some(code)` too)")
    // );

    SKIN.get_or_init(|| skin);
    println!("{}\n", logo());

    let mut app = App::from(&args.path, false)?;
    loop {
        let ids = app.retrieve()?;
        if ids.is_empty() {
            break;
        }

        println!("\n{} Totally {} items to review.", INFO.blue(), ids.len());
        if args.count_as_a_day {
            println!("{} Counting as a day.", INFO.blue());
        }
        let mut cnt = 0;

        app.count_as_a_day = args.count_as_a_day;
        let quit = logic::<_, anyhow::Error>(&mut app, |item| {
            let ratio = cnt as f64 / ids.len() as f64;
            print_progress_bar(ratio);
            let ans = handle_item(item);
            cnt += 1;
            ans
        })?;
        if quit || !args.repeat {
            let ratio = cnt as f64 / ids.len() as f64;
            print_progress_bar(ratio);
            break;
        }
    }

    println!("{} Saving progress...", INFO.blue());
    app.save(&args.path)?;
    println!("{} Bye bye.", INFO.blue());
    Ok(())
}

#[inline]
fn print_progress_bar(ratio: f64) {
    let finished = (100.0 * ratio).ceil() as usize;
    println!(
        "\n{}{}",
        (0..finished).map(|_| "#").collect::<Vec<&str>>().join(""),
        (0..if finished <= 100 { 100 - finished } else { 0 })
            .map(|_| "-")
            .collect::<Vec<&str>>()
            .join("")
    );
}

fn handle_item(item: &Item<Word>) -> anyhow::Result<Option<Quality>> {
    print!("[{}]", item.payload.word.bold().bright_green().italic());
    io::stdout().flush().unwrap();

    {
        let mut stdin_handle = io::stdin().lock();
        let mut byte = [0_u8];
        stdin_handle.read_exact(&mut byte).unwrap();
    }

    SKIN.get()
        .unwrap()
        .print_inline(item.payload.detail()?.as_str());
    println!("\n");
    println!(
        "{} Blackout(b) | Incorrect(i) | correct but Hard(h) | Correct(c) | perFect(f) | Never(n):",
        MARK.bright_purple()
    );
    read_quality()
}

fn read_quality() -> anyhow::Result<Option<Quality>> {
    print!("{} ", INPUT.bright_yellow());
    io::stdout().flush().unwrap();

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
        "n" => todo!(),
        "q" => return Ok(None),
        x => {
            eprintln!("unknown mark: {}", x);
            return read_quality();
        }
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

pub fn add(app: &mut App, name: String, path: String) -> io::Result<()> {
    fs::File::create(&path)?;
    app.add(Word {
        word: name,
        detail: path,
    });
    Ok(())
}

pub fn handle_delete(args: &Args, name: String) -> anyhow::Result<()> {
    let mut app = App::from(&args.path, false)?;
    app.db.mem.retain(|_, item| item.payload.word != name);
    let ans = app.save(&args.path)?;
    Ok(ans)
}
