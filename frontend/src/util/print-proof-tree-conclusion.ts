import { print_prop, ProofTreeConclusion } from 'alice';
import { printTypeJudgment } from './print-type-judgment';

export function printProofTreeConclusion(conclusion: ProofTreeConclusion): string {
    switch (conclusion.kind) {
        case 'PropIsTrue': return print_prop(conclusion.value);
        case 'TypeJudgement': return printTypeJudgment(conclusion.value);
    }
}