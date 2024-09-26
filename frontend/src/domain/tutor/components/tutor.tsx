import React from 'react';
import { TutorPropositionSolutionStatus, TutorPropositionSolutionStatusStatus } from './tutor-proposition-solution-status';
import { TutorGoalDisplay, TutorGoalDisplayGoal } from './tutor-goal-display';
import { get_free_parameters, has_quantifiers, ProofTerm, ProofTree, TypeCheckerGoal, VerificationResult } from 'alice';
import { TutorTypeCheckErrorDisplay } from './tutor-type-check-error-display';
import { TutorSyntaxErrorDisplay } from './tutor-syntax-error-display';
import { TutorProofPipelineErrorDisplay } from './tutor-proof-pipeline-error-display';

interface TutorProps {
    code: string;
    verificationResult?: VerificationResult;
}

export function Tutor(props: TutorProps) {
    const { code, verificationResult } = props;

    if (!verificationResult) {
        return;
    }

    return (
        <>
            <TutorPropositionSolutionStatus
                status={getStatus(verificationResult)}
                percentage={getProgress(verificationResult)}
            />
            <br />
            <hr style={{ borderColor: 'rgba(124, 178, 251, 0.25)' }} />
            <br />
            {
                (
                    verificationResult.kind === 'LexerError' ||
                    verificationResult.kind === 'ParserError'
                ) && (
                    <TutorSyntaxErrorDisplay errorMessage={verificationResult.value.error_message} />
                )
            }
            {
                verificationResult.kind === 'ProofPipelineError' && (
                    <TutorProofPipelineErrorDisplay error={verificationResult.value.error} />
                )
            }
            {
                verificationResult.kind === 'TypeCheckerError' && (
                    <TutorTypeCheckErrorDisplay
                        code={code}
                        error={verificationResult.value.error}
                    />
                )
            }
            {
                verificationResult.kind === 'TypeCheckSucceeded' && (
                    <TutorGoalDisplay
                        goals={transformGoalsForTutorGoalDisplay(verificationResult.value.result.goals)}
                    />
                )
            }
        </>
    );
}

function getStatus(verificationResult: VerificationResult): TutorPropositionSolutionStatusStatus {
    if (verificationResult.kind === 'TypeCheckSucceeded' && verificationResult.value.result.goals.length === 0) {
        return 'solved';
    }

    switch (verificationResult.value.solvable) {
        case 'Solvable': return 'solvable';
        case 'Unsolvable': return 'unsolvable';
        case 'Unknown': return 'unknown';
    }
}

function getProgress(verificationResult: VerificationResult): number {

    if (
        verificationResult.kind === 'LexerError' ||
        verificationResult.kind === 'ParserError' ||
        verificationResult.kind === 'ProofPipelineError' ||
        verificationResult.kind === 'TypeCheckerError'
    ) {
        return 0;
    }

    const goals = verificationResult.value.result.goals;

    if (goals.length === 0) {
        return 100;
    }

    if (goals.some((goal) => !goal.solution)) {
        return 0;
    }

    const proof_tree = verificationResult.value.result.proof_tree;
    const userSolutionDepth = getTreeDepth(proof_tree) - 1; // ignore initial node
    const goalSolutions: ProofTerm[] = goals.map((goal) => goal.solution!);
    const solutionDepth = Math.max(...goalSolutions.map(getProofTermDepth));

    return Math.floor(100 * userSolutionDepth / (userSolutionDepth + solutionDepth));
}

function getTreeDepth(proofTree: ProofTree): number {
    let maxChildDepth = 0;
    for (const premisse of proofTree.premisses) {
        const childDepth = getTreeDepth(premisse);

        if (childDepth > maxChildDepth) {
            maxChildDepth = childDepth;
        }
    }

    return 1 + maxChildDepth;
}

function getProofTermDepth(proofTerm: ProofTerm): number {
    switch (proofTerm.kind) {
        case 'Ident':
        case 'Unit':
        case 'Sorry':
            return 1;

        case 'Pair':
            return (
                1 + Math.max(getProofTermDepth(proofTerm.value[0]), getProofTermDepth(proofTerm.value[0]))
            );

        case 'ProjectFst':
        case 'ProjectSnd':
        case 'Abort':
        case 'OrLeft':
        case 'OrRight':
            return 1 + getProofTermDepth(proofTerm.value[0]);
        case 'TypeAscription':
            return 1 + getProofTermDepth(proofTerm.value.proof_term);

        case 'Function':
            return (
                1 + getProofTermDepth(proofTerm.value.body)
            );

        case 'Application':
            return (
                1 +
                Math.max(
                    getProofTermDepth(proofTerm.value.function),
                    getProofTermDepth(proofTerm.value.applicant)
                )
            );

        case 'LetIn':
            return (
                1 +
                Math.max(
                    getProofTermDepth(proofTerm.value.head),
                    getProofTermDepth(proofTerm.value.body)
                )
            );

        case 'Case':
            return (
                1 +
                Math.max(
                    getProofTermDepth(proofTerm.value.head),
                    getProofTermDepth(proofTerm.value.fst_term),
                    getProofTermDepth(proofTerm.value.snd_term)
                )
            );
    }
}

function transformGoalsForTutorGoalDisplay(goals: TypeCheckerGoal[]): TutorGoalDisplayGoal[] {
    return goals
        .map((goal) => ({
            hint: createHintFromGoal(goal),
            proofTreeConclusion: goal.conclusion,
        }));
}

function createHintFromGoal(goal: TypeCheckerGoal): string {
    const { solution, conclusion } = goal;

    if (!solution && conclusion.kind === 'PropIsTrue') {
        if (has_quantifiers(conclusion.value) || get_free_parameters(conclusion.value).length > 0) {
            return 'This branch contains propositions in first-order logic. Alice can\'t assist you with that. Sorry :(';
        }

        return 'Alice couldn\'t find a solution for this branch. You might want to check your current proof.';
    }

    if (!solution && conclusion.kind === 'TypeJudgement') {
        return 'Alice can\'t assist you with that goal.';
    }

    if (!solution) {
        throw new Error('Solution is null.');
    }

    switch (solution.kind) {
        case 'Ident': return `Use identifier ${solution.value[0]}.`;
        case 'Pair': return `Use rule ∧I.`;
        case 'ProjectFst': return `Use rule ∧E₁.`;
        case 'ProjectSnd': return `Use rule ∧E₂.`;
        case 'Function': return `Use rule ⊃I.`;
        case 'Application': return `Use rule ⊃E.`;
        case 'LetIn': return `Use rule ∃E.`;
        case 'OrLeft': return `Use rule ∨I₁.`;
        case 'OrRight': return `Use rule ∨I₂.`;
        case 'Case': return `Use rule ∨E.`;
        case 'Abort': return `Use rule ⊥E.`;
        case 'Unit': return `Use rule ⊤I`;
    }

    throw new Error('Prover returned strange solution: ' + JSON.stringify(solution));
}