import { VisualProofEditorProofTree } from '../../components/visual-proof-editor';
import { ProofRuleHandlerResult } from '../../components/visual-proof-editor-sidebar';
import { createEmptyVisualProofEditorProofTree } from '../../../../util/create-visual-proof-editor-empty-proof-tree';

export function handleOrIntroFstRule(proofTree: VisualProofEditorProofTree): ProofRuleHandlerResult {
    const { id, conclusion } = proofTree;

    if (conclusion.kind != 'Or') {
        throw new Error('Conclusion is not a disjunction.');
    }

    const [fst, _snd] = conclusion.value;

    return {
        additionalAssumptions: [],
        newProofTree: {
            id: id,
            premisses: [createEmptyVisualProofEditorProofTree(fst)],
            rule: 'OrIntroFst',
            conclusion,
        },
    };
}