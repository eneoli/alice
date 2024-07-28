import Swal from 'sweetalert2';
import { VisualProofEditorRuleHandlerParams, ProofRuleHandlerResult } from '..';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../../../util/create-visual-proof-editor-empty-proof-tree';
import { ProofRuleHandler } from './proof-rule-handler';
import { v4 } from 'uuid';

export class FalseElimRuleHandler extends ProofRuleHandler {
    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const { selectedProofTreeNodes: selecteedProofTreeNodes } = params;

        if (selecteedProofTreeNodes.length !== 1) {
            throw new Error('Cannot apply this rule on multiple nodes.');
        }

        const { proofTree, reasoningContextId } = selecteedProofTreeNodes[0];

        return {
            additionalAssumptions: [],
            removedReasoingContextIds: [],
            newReasoningContexts: [],
            proofTreeChanges: [{
                newProofTree: {
                    ...proofTree,
                    premisses: [createEmptyVisualProofEditorProofTreeFromProp({ kind: 'False' })],
                    rule: { kind: 'FalsumElim' },
                },
                reasoningContextId,
                nodeId: proofTree.id
            }]
        };
    }

    protected async handleRuleDownards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const { selectedProofTreeNodes: selecteedProofTreeNodes } = params;

        if (selecteedProofTreeNodes.length !== 1) {
            throw new Error('Cannot apply this rule on multiple nodes.');
        }

        const { proofTree, reasoningContextId } = selecteedProofTreeNodes[0];
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            throw new Error('Conclusion is not ⊥.');
        }

        const conclusionProp = conclusion.value;

        if (conclusionProp.kind !== 'False') {
            throw new Error('Conclusion is not ⊥.');
        }

        const newConclusionResult = await Swal.fire({
            title: 'Enter new conclusion.',
            input: 'text',
            inputPlaceholder: 'A',
            showCloseButton: true,
        });

        let newConclusion = newConclusionResult.value;

        if (!newConclusion) {
            return this.createEmptyProofRuleHandlerResult();
        }

        newConclusion = this.parseProp(newConclusion);

        return {
            additionalAssumptions: [],
            removedReasoingContextIds: [],
            newReasoningContexts: [],
            proofTreeChanges: [{
                newProofTree: {
                    id: v4(),
                    premisses: [proofTree],
                    rule: { kind: 'FalsumElim' },
                    conclusion: { kind: 'PropIsTrue', value: newConclusion },
                },
                nodeId: proofTree.id,
                reasoningContextId,
            }],
        };
    }
}
