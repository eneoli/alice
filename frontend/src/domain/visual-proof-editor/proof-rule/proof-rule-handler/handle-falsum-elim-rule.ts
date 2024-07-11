import { createEmptyVisualProofEditorProofTreeFromProp } from '../../../../util/create-visual-proof-editor-empty-proof-tree';
import { VisualProofEditorProofTree } from '../../components/visual-proof-editor';
import { ProofRuleHandlerResult } from '../../components/visual-proof-editor-sidebar';

export async function handleFalsumElimRule(proofTree: VisualProofEditorProofTree): Promise<ProofRuleHandlerResult> {

    const {id, conclusion} = proofTree;

    return {
        additionalAssumptions: [],
        newProofTree: {
            id,
            premisses: [
                createEmptyVisualProofEditorProofTreeFromProp({ kind: 'False' }),
            ],
            rule: { kind: 'FalsumElim' },
            conclusion,
        },
    };
}