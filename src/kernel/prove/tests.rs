#[cfg(test)]
mod tests {
    use chumsky::{primitive::end, Parser, Stream};

    use crate::kernel::{
        checker::{check::check, identifier_context::IdentifierContext},
        parse::{fol::fol_parser, lexer::lexer}, prove::prove,
    };

    pub fn assert_proof(prop: &str) {
        let prop_tokens = lexer().then_ignore(end()).parse(prop).unwrap();

        let prop_len = prop.chars().count();

        let prop = fol_parser()
            .parse(Stream::from_iter(
                prop_len..prop_len + 1,
                prop_tokens.into_iter(),
            ))
            .unwrap();

        let proof_term_result = prove(&prop);

        assert!(proof_term_result.is_some());
        assert!(check(
            &proof_term_result.unwrap(),
            &prop,
            &IdentifierContext::new()
        )
        .is_ok());
    }

    pub fn assert_no_proof(prop: &str) {
        let prop_tokens = lexer().then_ignore(end()).parse(prop).unwrap();

        let prop_len = prop.chars().count();

        let prop = fol_parser()
            .parse(Stream::from_iter(
                prop_len..prop_len + 1,
                prop_tokens.into_iter(),
            ))
            .unwrap();

        let proof_term_result = prove(&prop);

        assert!(proof_term_result.is_none());
    }

    #[test]
    fn test_no_falsum_proof() {
        assert_no_proof("\\bot");
    }

    #[test]
    fn test_truth_proof() {
        assert_proof("\\top");
    }

    #[test]
    fn test_atom_identity() {
        assert_proof("A -> A");
    }

    #[test]
    fn test_impl_with_unused() {
        assert_proof("A -> (B -> A)");
    }

    #[test]
    fn test_impl_none() {
        assert_no_proof("A -> B");
    }

    #[test]
    fn test_swap() {
        assert_proof("A & B -> B & A");
    }

    #[test]
    fn test_currying() {
        assert_proof("(A & B -> C) -> (A -> B -> C)");
    }

    #[test]
    fn test_uncurrying() {
        assert_proof("(A -> B -> C) -> (A & B -> C)");
    }

    #[test]
    fn test_project_fst() {
        assert_proof("A && B -> A");
    }

    #[test]
    fn test_project_snd() {
        assert_proof("A && B -> B");
    }

    #[test]
    fn test_impl_switch_params() {
        assert_proof("(A -> B -> C) -> (B -> A -> C)");
    }

    #[test]
    fn test_impl_elim() {
        assert_proof("A && (A -> B) -> B");
    }

    #[test]
    fn test_interaction_law_of_distributivity() {
        assert_proof("(A -> (B & C)) -> ((A -> B) & (A -> C))");
    }

    #[test]
    fn test_distribution_of_or() {
        assert_proof("A -> (B || C) -> ((A -> B) || (A -> C))");
    }

    #[test]
    fn test_undo_distribution_of_or() {
        assert_proof("((A -> B) || (A -> C)) -> A -> (B || C)");
    }

    #[test]
    fn test_commutativity_of_disjunction() {
        assert_proof("A || B -> B || A");
    }

    #[test]
    fn test_ex_falso_quodlibet() {
        assert_proof("\\bot -> A");
    }

    #[test]
    fn test_composition_of_functions() {
        assert_proof("((A -> B) & (B -> C)) -> (A -> C)");
    }

    #[test]
    fn test_apply_composition_of_functions() {
        assert_proof("A && (A -> B) && (B -> C) -> C");
    }

    #[test]
    fn test_s_combinator() {
        assert_proof("(A -> B) -> (A -> (B -> C)) -> (A -> C)");
    }

    #[test]
    fn test_no_negated_lem() {
        assert_no_proof("~(A || ~A)");
    }

    #[test]
    fn test_lem_double_negated() {
        assert_proof("~~(A || ~A)");
    }

    #[test]
    fn test_tripple_negation_elimination() {
        assert_proof("~~~A -> ~A");
    }

    #[test]
    fn test_no_double_negation_elimination() {
        assert_no_proof("~~A -> A");
    }

    #[test]
    fn test_or_impl() {
        assert_proof("A || B -> B || A -> B || A -> A || B");
    }

    #[test]
    fn test_peirces_law() {
        assert_no_proof("((A -> B) -> A) -> A");
    }

    #[test]
    fn test_double_negation_peirces_law() {
        assert_proof("~~(((A -> B) -> A) -> A)");
    }

    #[test]
    fn test_long_proof() {
        assert_proof("(P -> (C ∧ K) || (D ∧ L)) -> (~K -> S) -> (D || L) -> (P -> ~S) ∧ (S -> ~P) -> (C -> ~D) ∧ (D -> ~C) -> (K -> ~L) ∧ (L -> ~K) -> ~P");
    }

    #[test]
    fn test_impl_and_or() {
        assert_proof("(A || C) && (B -> C) -> (A -> B) -> C");
    }

    #[test]
    fn test_some_tautologies() {
        assert_proof("((A || B) && (B -> C)) -> ((A -> B) -> C)");
        assert_proof("(A || B -> C) -> (A -> C) && (B -> C)");
        assert_proof("(A -> C) && (B -> C) -> (A || B -> C)");

        assert_proof("(A && (B || C)) -> (A && B) || (A && C)");
        assert_proof("(A && B) || (A && C) -> (A && (B || C))");

        assert_proof("(A || (B && C)) -> (A || B) && (A || C)");
        assert_proof("(A || B) && (A || C) -> (A || (B && C))");

        assert_proof("((A -> B) -> (A -> C) -> A -> B)");
        assert_proof("((A -> B) -> (A -> C) -> A -> C)");
        assert_proof("A -> ((A -> B) -> (A -> C) -> B)");
        assert_proof("A -> ((A -> B) -> (A -> C) -> C)");
        assert_proof("((A -> B -> C) -> A -> B -> C)");
        assert_proof("((A -> B -> C) -> B -> A -> C)");
        assert_proof("A -> B -> (A -> B -> C) -> C");
        assert_proof("B -> A -> (A -> B -> C) -> C");

        assert_proof("((A -> B) -> A -> B)");
        assert_proof("(((A -> B) -> C) -> ((A -> B) -> C))");
        assert_proof("((((A -> B) -> C) -> D) -> (((A -> B) -> C) -> D))");
        assert_proof("(((((A -> B) -> C) -> D) -> E) -> (((A -> B) -> C) -> D) -> E)");
        assert_proof("((((((A -> B) -> C) -> D) -> E) -> F) -> (((A -> B) -> C) -> D) -> E -> F)");
        assert_proof("((((((A -> B) -> C) -> D) -> E) -> F) -> (((((A -> B) -> C) -> D) -> E) -> F) || (((((A -> B) -> C) -> D) -> E) -> F))");
        assert_proof("((A -> B) -> C) -> D -> D || D");
    }

    #[test]
    fn test_demorgan() {
        assert_no_proof("~(A && B) -> ~A || ~B");
        assert_proof("~A || ~B -> ~(A && B)");

        assert_proof("~(A || B) -> ~A && ~B");
        assert_proof("~A && ~B -> ~(A || B)");

        assert_no_proof("~(A -> B) -> A && ~B");
        assert_proof("A && ~B -> ~(A -> B)");

        assert_proof("A -> ~~A");

        assert_proof("~\\top -> \\bot");
        assert_proof("\\bot -> ~\\top");
    }

    #[test]
    fn test_no_proofs() {
        assert_no_proof("(A -> B || C) -> (A -> B) || (A -> C)");
        assert_no_proof("((A -> B) -> C) -> ((A || B) && (B -> C))");
    }

    #[test]
    fn test_three_way_composition() {
        assert_proof("(A -> B) -> (B -> C) -> (C -> D) -> (A -> D)");
    }
}