import { VisualProofEditorProofTree } from '../../components/visual-proof-editor';
import { ProofRuleHandlerResult } from '../../components/visual-proof-editor-sidebar';
import { createEmptyVisualProofEditorProofTree } from '../../../../util/create-visual-proof-editor-empty-proof-tree';

export function handleOrIntroSndRule(proofTree: VisualProofEditorProofTree): ProofRuleHandlerResult {
    const { id, conclusion } = proofTree;

    if (conclusion.kind != 'Or') {
        throw new Error('Conclusion is not a disjunction.');
    }

    const [_fst, snd] = conclusion.value;

    return {
        newProofTree: {
            id,
            premisses: [createEmptyVisualProofEditorProofTree(snd)],
            rule: 'OrIntroSnd',
            conclusion,
        },
        additionalAssumptions: [],
    }
}
