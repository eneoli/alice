import { VisualProofEditorProofTree } from '../../components/visual-proof-editor';
import { ProofRuleHandlerResult } from '../../components/visual-proof-editor-sidebar';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../../../util/create-visual-proof-editor-empty-proof-tree';

export async function handleAndIntroRule(proofTree: VisualProofEditorProofTree): Promise<ProofRuleHandlerResult> {
    const { rule, conclusion, id } = proofTree;

    if (conclusion.kind !== 'PropIsTrue') {
        throw new Error('Conclusion is not a conjunction');
    }

    const propConclusion = conclusion.value;

    if (propConclusion.kind !== 'And') {
        throw new Error('Conclusion is not a conjunction');
    }

    if (rule !== null) {
        throw new Error('Cannot reason upwards.');
    }

    const [fst, snd] = propConclusion.value;

    return {
        additionalAssumptions: [],
        newProofTree: {
            id,
            premisses: [
                createEmptyVisualProofEditorProofTreeFromProp(fst),
                createEmptyVisualProofEditorProofTreeFromProp(snd),
            ],
            rule: { kind: 'AndIntro' },
            conclusion,
        },
    };
}
