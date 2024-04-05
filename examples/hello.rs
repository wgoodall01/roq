use roq_derive::definition;

fn main() {
    println!("double(21) = {}", double(21));
}

#[definition]
fn double(a: u64) -> u64 {
    a + a
}
