import { VisualProofEditorRuleHandlerParams, ProofRuleHandlerResult } from '..';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../../../util/create-visual-proof-editor-empty-proof-tree';

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