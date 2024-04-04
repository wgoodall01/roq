use roq_derive::definition;

fn main() {
    println!("Hello, World!");
}

#[definition(generate = "double.v")]
fn double(a: u64) -> u64 {
    a + a
}
