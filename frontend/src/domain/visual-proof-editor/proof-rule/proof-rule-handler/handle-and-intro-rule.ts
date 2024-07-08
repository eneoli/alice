import { VisualProofEditorProofTree } from '../../components/visual-proof-editor';
import { ProofRuleHandlerResult } from '../../components/visual-proof-editor-sidebar';
import { createEmptyVisualProofEditorProofTree } from '../../../../util/create-visual-proof-editor-empty-proof-tree';

export async function handleAndIntroRule(proofTree: VisualProofEditorProofTree): Promise<ProofRuleHandlerResult> {
    const { rule, conclusion, id } = proofTree;

    if (conclusion.kind != 'And') {
        throw new Error('Conclusion is not a conjunction');
    }

    if (rule !== null) {
        throw new Error('Cannot reason upwards.');
    }

    const [fst, snd] = conclusion.value;

    return {
        additionalAssumptions: [],
        newProofTree: {
            id,
            premisses: [
                createEmptyVisualProofEditorProofTree(fst),
                createEmptyVisualProofEditorProofTree(snd),
            ],
            rule: { kind: 'AndIntro' },
            conclusion,
        },
    };
}
