import { createEmptyVisualProofEditorProofTree } from '../../../../util/create-visual-proof-editor-empty-proof-tree';
import { VisualProofEditorProofTree } from '../../components/visual-proof-editor';
import { ProofRuleHandlerResult } from '../../components/visual-proof-editor-sidebar';

export function handleFalsumElimRule(proofTree: VisualProofEditorProofTree): ProofRuleHandlerResult {

    const {id, conclusion} = proofTree;

    return {
        additionalAssumptions: [],
        newProofTree: {
            id,
            premisses: [
                createEmptyVisualProofEditorProofTree({ kind: 'False' }),
            ],
            rule: { kind: 'FalsumElim' },
            conclusion,
        },
    };
}