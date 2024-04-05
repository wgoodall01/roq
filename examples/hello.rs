use roq_derive::definition;

fn main() {
    println!("double(21) = {}", double(21));

    // Get the Coq vernacular for this definition.
    let defn = double::roq::as_definition();
    println!("Coq < {defn}");
}

#[definition]
fn double(a: u64) -> u64 {
    a + a
}
