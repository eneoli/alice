import { createEmptyVisualProofEditorProofTreeFromProp } from '../../../../util/create-visual-proof-editor-empty-proof-tree';
import { ProofRuleHandlerResult, VisualProofEditorRuleHandlerParams } from '../../components/visual-proof-editor-sidebar';

export async function handleFalsumElimRule({ proofTree }: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {

    const { id, conclusion } = proofTree;

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