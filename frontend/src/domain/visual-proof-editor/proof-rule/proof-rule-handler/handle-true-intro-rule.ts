import { VisualProofEditorProofTree } from '../../components/visual-proof-editor';
import { ProofRuleHandlerResult } from '../../components/visual-proof-editor-sidebar';

export function handleTrueIntroRule(proofTree: VisualProofEditorProofTree): ProofRuleHandlerResult {
    const { conclusion } = proofTree;

    if (conclusion.kind !== 'True') {
        throw new Error('Conclusion is not truth.');
    }

    return {
        additionalAssumptions: [],
        newProofTree: {
            ...proofTree,
            rule: { kind: 'TrueIntro' },
        }
    };
}