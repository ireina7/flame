use anyhow::anyhow;
use clap::Parser;
use colored::Colorize;
use driver::Command;
use words::App;

mod driver;
mod review;
mod words;

fn main() {
    let args = driver::Args::parse();

    let ans = match &args.cmd {
        Command::Up => driver::run(args),
        Command::Add { name, path } => handle_add(&args, name.to_owned(), path.to_owned()),
        Command::Delete { name } => handle_delete(&args, name.to_owned()),
        Command::Show { name } => handle_show(&args, name.to_owned()),
        Command::Clear => unimplemented!(),
    };

    if let Err(err) = ans {
        panic!("{}", err)
    }
}

fn handle_show(args: &driver::Args, name: String) -> anyhow::Result<()> {
    let app = App::from(&args.path, false)?;
    let item = app
        .db
        .mem
        .iter()
        .find(|(_, item)| item.payload.word == name)
        .map(|(_, item)| item.clone())
        .ok_or(anyhow!("word not found"))?;
    println!("{} {:?}", driver::INFO.blue(), item);
    Ok(())
}

fn handle_delete(args: &driver::Args, name: String) -> anyhow::Result<()> {
    let mut app = App::from(&args.path, false)?;
    app.db.mem.retain(|_, item| item.payload.word != name);
    let ans = app.save(&args.path)?;
    Ok(ans)
}

fn handle_add(args: &driver::Args, name: String, path: String) -> anyhow::Result<()> {
    let mut app = App::from(&args.path, false)?;
    driver::add(&mut app, name.to_string(), path.to_string())?;
    let ans = app.save(&args.path)?;
    Ok(ans)
}
