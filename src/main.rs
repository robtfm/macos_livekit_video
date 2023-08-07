pub mod signed_login;
pub mod test;
pub mod wallet;

fn main() {
    println!("running no_video");
    test::no_video();
    println!("running video");
    test::with_video();
}
