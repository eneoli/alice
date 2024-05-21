use crate::core::{Prop, Sequent};
use std::ops::Deref;

pub fn prove(prop: Prop) -> bool {
    let sequent = Sequent::new(&prop);

    prove_right(sequent)
}

fn prove_right(sequent: Sequent) -> bool {
    use Prop::*;

    match sequent.goal {
        Atom(_) => sequent.ctx_contains(&sequent.goal) || prove_left(sequent),

        And(ref left, ref right) => {
            prove_right(sequent.with_new_goal(left)) && prove_right(sequent.with_new_goal(right))
        }

        Or(_, _) => prove_left(sequent),

        Implication(ref left, ref right) => {
            let mut seq = sequent.with_new_goal(right);
            seq.push_inv(left);

            prove_right(seq)
        }

        True => true,

        False => prove_left(sequent),
    }
}

fn prove_left(mut sequent: Sequent) -> bool {
    use Prop::*;

    let inv_prop = sequent.pop_inv();

    // we have an inversable prop
    if let Some(prop) = inv_prop {
        return match prop {
            Atom(_) => {
                (sequent.goal == prop) || {
                    sequent.add_non_inv(&prop);
                    prove_left(sequent)
                }
            }

            And(ref left, ref right) => {
                sequent.push_inv(left);
                sequent.push_inv(right);

                prove_left(sequent)
            }

            Or(ref left, ref right) => {
                let mut left_goal = sequent.clone();
                let mut right_goal = sequent.clone();

                left_goal.push_inv(left);

                right_goal.push_inv(right);

                prove_left(left_goal) && prove_left(right_goal)
            }

            Implication(_, _) => {
                sequent.add_non_inv(&prop);

                prove_left(sequent)
            }

            True => prove_left(sequent),

            False => true,
        };
    }

    // proof search in unordered context
    // try remaining rules

    // id rule
    if sequent.ctx_contains(&sequent.goal) {
        return true;
    }

    // OR rules
    if let Or(ref left, ref right) = sequent.goal {
        // 1. OR rule
        let first_sequent = sequent.with_new_goal(left);
        if prove_right(first_sequent) {
            return true;
        }

        // 2. OR rule
        let second_sequent = sequent.with_new_goal(right);
        if prove_right(second_sequent) {
            return true;
        }
    }

    for (i, non_inv_prop) in sequent.non_inv_ctx.iter().enumerate() {
        if let Implication(ref left, ref right) = non_inv_prop {
            if *left.deref() == sequent.goal {
                continue;
            }

            let left_goal = sequent.with_new_goal(left);
            let mut right_goal = sequent.clone();
            right_goal.push_inv(right);
            right_goal.non_inv_ctx.remove(i);

            return prove_right(left_goal) && prove_left(right_goal);
        }
    }

    // we failed :(
    false
}
