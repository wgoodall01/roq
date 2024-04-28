use roq_derive::definition;

#[definition]
fn max(a: u64, b: u64) -> u64 {
    // Check if `a` is less than `b`
    let a_lt_b = a < b;

    // If `a` is less than `b`, return `b`, otherwise return `a`
    if a_lt_b {
        b
    } else {
        a
    }
}

#[test]
fn test_max() {
    assert_eq!(max(1, 2), 2);
    assert_eq!(max(2, 1), 2);
    assert_eq!(max(0, 10), 10);
}

/// Prove that `max(2, n) >= 2`
#[test]
fn prove_max() {
    roq::prove! {
        function max,
        inline r"
            Require Import Lia.
            Require Import Arith.
            Theorem clamp : forall (n: nat),
              max 2 n >= 2.
            Proof.
              intros.
              unfold max.
              destruct (Nat.ltb 2 n) eqn: H;
              try apply Nat.ltb_lt in H;
              lia.
            Qed.
        "
    };
}
