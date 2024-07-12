import { ProofRuleHandlerResult, VisualProofEditorRuleHandlerParams } from '../../components/visual-proof-editor-sidebar';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../../../util/create-visual-proof-editor-empty-proof-tree';

export async function handleOrIntroFstRule({ proofTree }: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
    const { id, conclusion } = proofTree;

    if (conclusion.kind !== 'PropIsTrue') {
        throw new Error('Conclusion is not a disjunction.');
    }

    const propConclusion = conclusion.value;

    if (propConclusion.kind !== 'Or') {
        throw new Error('Conclusion is not a disjunction.');
    }

    const [fst, _snd] = propConclusion.value;

    return {
        additionalAssumptions: [],
        newProofTree: {
            id: id,
            premisses: [createEmptyVisualProofEditorProofTreeFromProp(fst)],
            rule: { kind: 'OrIntroFst' },
            conclusion,
        },
    };
}
