use chumsky::prelude::*;

use crate::core::proof_term::ProofTerm;

use super::{fol::fol_parser, Token};

/*
    == Proof Term Parser ==

    Expr           = Function | Case | Application ;
    Unit           = "(", ")" ;
    Pair           = "(", Expr, ",", Expr, ")" ;
    Atom           = "(", Expr, ")" | Ident | Pair | Unit ;
    Function       = "fn", Ident, ":", "(", Prop, ")", "=>", Expr ;
    CaseExpr       = Case | Application ;
    Case           = "case", CaseExpr, "of", "inl", Ident, "=>", Expr, ",", "inr", Ident, "=>", Expr, [","] ;
    Application    = Atom, {Atom | Function | Case} ;
*/
pub fn proof_term_parser() -> impl Parser<Token, ProofTerm, Error = Simple<Token>> {
    let ident_token = select! { Token::IDENT(ident) => ident };

    let proof_term = recursive(|proof_term| {
        let ident_term = ident_token.map(ProofTerm::Ident).boxed();

        let unit = just(Token::LROUND)
            .then(just(Token::RROUND))
            .map(|_| ProofTerm::Unit)
            .boxed();

        let pair = just(Token::LROUND)
            .ignore_then(proof_term.clone())
            .then_ignore(just(Token::COMMA))
            .then(proof_term.clone())
            .then_ignore(just(Token::RROUND))
            .map(|(fst, snd)| ProofTerm::Pair(Box::new(fst), Box::new(snd)))
            .boxed();

        let atom = choice((
            proof_term
                .clone()
                .delimited_by(just(Token::LROUND), just(Token::RROUND)),
            ident_term.clone(),
            pair.clone(),
            unit.clone(),
        ))
        .boxed();

        let function = just(Token::FN)
            .ignore_then(ident_token)
            .then_ignore(just(Token::COLON))
            .then(fol_parser().delimited_by(just(Token::LROUND), just(Token::RROUND)))
            .then_ignore(just(Token::IMPLICATION))
            .then(proof_term.clone())
            .map(|((param_ident, param_prop), body)| ProofTerm::Function {
                param_ident,
                param_prop,
                body: Box::new(body),
            })
            .boxed();

        let case = |application: Recursive<'static, Token, ProofTerm, Simple<Token>>| {
            recursive(|case| {
                let case_expr = choice((case.clone(), application.clone()));

                just(Token::CASE)
                    .ignore_then(case_expr.clone())
                    .then_ignore(just(Token::OF))
                    //
                    .then_ignore(just(Token::IDENT("inl".to_string())))
                    .then(ident_token.clone())
                    .then_ignore(just(Token::IMPLICATION))
                    .then(proof_term.clone())
                    .then_ignore(just(Token::COMMA))
                    //
                    .then_ignore(just(Token::IDENT("inr".to_string())))
                    .then(ident_token.clone())
                    .then_ignore(just(Token::IMPLICATION))
                    .then(proof_term.clone())
                    .then_ignore(just(Token::COMMA).or_not())
                    .map(
                        |((((proof_term, left_ident), left_term), right_ident), right_term)| {
                            ProofTerm::Case {
                                proof_term: Box::new(proof_term),
                                left_ident,
                                left_term: Box::new(left_term),
                                right_ident,
                                right_term: Box::new(right_term),
                            }
                        },
                    )
                    .boxed()
            })
        };

        let application = recursive(|application| {
            atom.clone()
                .then(
                    choice((atom.clone(), function.clone(), case(application).clone())).repeated(),
                )
                .foldl(|lhs, rhs| {
                    if let ProofTerm::Ident(ident) = lhs.clone() {
                        match ident.as_str() {
                            "inl" => ProofTerm::OrLeft(Box::new(rhs)),
                            "inr" => ProofTerm::OrRight(Box::new(rhs)),
                            "abort" => ProofTerm::Abort(Box::new(rhs)),
                            _ => ProofTerm::Application {
                                function: Box::new(lhs),
                                applicant: Box::new(rhs),
                            },
                        }
                    } else {
                        ProofTerm::Application {
                            function: Box::new(lhs),
                            applicant: Box::new(rhs),
                        }
                    }
                })
                .boxed()
        });

        choice((function, case(application.clone()), application)).boxed()
    });

    proof_term.then_ignore(end())
}
