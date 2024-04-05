use roq_derive::definition;

fn main() {
    println!("double(21) = {}", double(21));
}

#[definition]
fn double(a: u64) -> u64 {
    a + a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double() {
        assert_eq!(double(1), 2);
    }

    /// Prove that `double(n) = 2 * n`
    #[test]
    fn prove_double() {
        roq::prove! {
            function double,
            inline r"
                Theorem double_mul : forall n : nat,
                  double n = 2 * n.
                Proof.
                  intros.
                  unfold double.
                  simpl.
                  rewrite <- plus_n_O.
                  reflexivity.
                Qed.
            "
        };
    }
}
