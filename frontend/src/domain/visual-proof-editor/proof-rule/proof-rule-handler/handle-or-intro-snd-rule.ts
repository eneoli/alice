import { VisualProofEditorRuleHandlerParams, ProofRuleHandlerResult } from '..';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../../../util/create-visual-proof-editor-empty-proof-tree';

export async function handleOrIntroSndRule({ proofTree }: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
    const { id, conclusion } = proofTree;

    if (conclusion.kind !== 'PropIsTrue') {
        throw new Error('Conclusion is not a disjunction.');
    }

    const propConclusion = conclusion.value;

    if (propConclusion.kind !== 'Or') {
        throw new Error('Conclusion is not a disjunction.');
    }

    const [_fst, snd] = propConclusion.value;

    return {
        newProofTree: {
            id,
            premisses: [createEmptyVisualProofEditorProofTreeFromProp(snd)],
            rule: { kind: 'OrIntroSnd' },
            conclusion,
        },
        additionalAssumptions: [],
    }
}
