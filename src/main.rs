mod driver;
mod review;
mod words;

fn main() {
    let ans = driver::run();
    if let Err(err) = ans {
        panic!("{}", err)
    }
}
