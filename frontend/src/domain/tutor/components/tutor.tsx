import React from 'react';
import { TutorPropositionSolutionStatus, TutorPropositionSolutionStatusStatus } from './tutor-proposition-solution-status';
import { TutorGoalDisplay, TutorGoalDisplayGoal } from './tutor-goal-display';
import { CheckError, ProofTerm, ProofTree, TypeCheckerGoal, TypeCheckerResult } from 'alice';
import { TutorTypeCheckErrorDisplay } from './tutor-type-check-error-display';

interface TutorProps {
    code: string;
    checkResult?: TypeCheckerResult;
    checkError?: CheckError;
}

export function Tutor(props: TutorProps) {
    const { code, checkResult, checkError } = props;

    return (
        <>
            <TutorPropositionSolutionStatus
                status={getStatus(checkResult)}
                percentage={checkResult ? getProgress(checkResult) : 0}
            />
            <br />
            <hr style={{ borderColor: 'rgba(124, 178, 251, 0.25)' }} />
            <br />
            {
                checkError && (
                    <TutorTypeCheckErrorDisplay
                        code={code}
                        error={checkError}
                    />
                )
            }
            <br />
            {
                checkResult && (
                    <TutorGoalDisplay
                        goals={transformGoalsForTutorGoalDisplay(checkResult?.goals)}
                    />
                )
            }
        </>
    );
}

function getStatus(checkResult: TypeCheckerResult | undefined): TutorPropositionSolutionStatusStatus {

    if (checkResult && checkResult.goals.length === 0) {
        return 'solved';
    }

    if (checkResult && checkResult.goals.every((goal) => goal.solution)) {
        return 'solvable';
    }

    return 'unknown';
}

function getProgress(checkResult: TypeCheckerResult): number {
    const userSolutionDepth = getTreeDepth(checkResult.proof_tree);
    const solutions: ProofTerm[] = checkResult.goals
        .map((goal) => goal.solution)
        .filter((solution) => solution !== null);

    const solutionDepth = Math.max(
        0,
        ...solutions.map(getProofTermDepth)
    );

    if (solutionDepth === 0 && checkResult.goals.length > 0) {
        return 0;
    }

    return Math.round(100 * userSolutionDepth / (userSolutionDepth + solutionDepth));
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
        return 'This branch seems to be not solvable. You might want to check your current proof.';
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