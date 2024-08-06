import Swal from 'sweetalert2';
import { VisualProofEditorRuleHandlerParams, ProofRuleHandlerResult } from '..';
import { ProofRuleHandler } from './proof-rule-handler';
import { v4 } from 'uuid';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../lib/visual-proof-editor-proof-tree';

export class FalseElimRuleHandler extends ProofRuleHandler {

    public getLatexCode(): string {
        return `
            \\begin{prooftree}
                \\AxiomC{$\\bot$}
                \\RightLabel{$\\bot E$}
                \\UnaryInfC{$C$}
            \\end{prooftree}
        `;
    }

    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined> {
        const { selectedProofTreeNodes, error } = params;

        if (selectedProofTreeNodes.length !== 1) {
            error('Cannot apply this rule on multiple nodes.');
            return;
        }

        const { proofTree, reasoningContextId } = selectedProofTreeNodes[0];

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

    protected async handleRuleDownards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined> {
        const { selectedProofTreeNodes, error } = params;

        if (selectedProofTreeNodes.length !== 1) {
            error('Cannot apply this rule on multiple nodes.');
            return;
        }

        const { proofTree, reasoningContextId } = selectedProofTreeNodes[0];
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            error('Conclusion is not ⊥.');
            return;
        }

        const conclusionProp = conclusion.value;

        if (conclusionProp.kind !== 'False') {
            error('Conclusion is not ⊥.');
            return;
        }

        const newConclusionResult = await Swal.fire({
            title: 'Enter new conclusion.',
            input: 'text',
            inputPlaceholder: 'A',
            showCloseButton: true,
        });

        let newConclusion = newConclusionResult.value;

        if (!newConclusion) {
            return;
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
