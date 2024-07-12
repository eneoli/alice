import { v4 } from 'uuid';
import { ProofRuleHandlerResult, VisualProofEditorRuleHandlerParams } from '../../components/visual-proof-editor-sidebar';

export async function handleAndElimSndRule({proofTree}: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
    const { conclusion } = proofTree;

    if (conclusion.kind !== 'PropIsTrue') {
        throw new Error('Conclusion is not a conjunction');
    }

    const propConclusion = conclusion.value;

    if (propConclusion.kind !== 'And') {
        throw new Error('Conclusion is not a conjunction');
    }

    const [_fst, snd] = propConclusion.value;

    return {
        additionalAssumptions: [],
        newProofTree: {
            id: v4(),
            premisses: [proofTree],
            rule: { kind: 'AndElimSnd' },
            conclusion: { kind: 'PropIsTrue', value: snd }
        }
    };
}