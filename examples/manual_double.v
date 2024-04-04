Require Import Lia.

Definition double (n: nat) : nat :=
  n + n.

(* Prove double(n) = n + n *)
Theorem double_plus : forall n : nat,
  double n = 2 * n.
Proof.
  intros.
  unfold double.
  simpl.
  rewrite <- plus_n_O.
  reflexivity.
Qed.
