// Checkable terms:     M, N ::= (M, N) | (fn u => M) | inl M | inr N | (case R of inl u => M, inr v => N) | abort R | () | R // We can check for a GIVEN Prop A if the term has this type
// Synthesizing terms:  R    ::= fst R | snd R | u | R M    // We either can infer exactly one Prop A (not given before) that the term has as type or there is no such A.
// Questions:
// 1. Why is () not a synthesizing term? It clearly always has type True
// 2. (R, R) would also be a synthesizing term. Why is it not in the list?

pub mod identifier_context;
pub mod check;
pub mod synthesize;

#[cfg(test)]
mod tests;