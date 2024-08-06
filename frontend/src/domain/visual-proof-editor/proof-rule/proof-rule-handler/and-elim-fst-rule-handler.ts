import { v4 } from 'uuid';
import { ProofRuleHandlerResult, VisualProofEditorRuleHandlerParams } from '..';
import Swal from 'sweetalert2';
import { ProofRuleHandler } from './proof-rule-handler';
import { Prop } from 'alice';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../lib/visual-proof-editor-proof-tree';

export class AndElimFstRuleHandler extends ProofRuleHandler {

    public getLatexCode(): string {
        return `
            \\begin{prooftree}
                \\AxiomC{$A \\land B$}
                \\RightLabel{$\\land E_1$}
                \\UnaryInfC{$A$}
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
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            error('Cannot apply rule on this node.');
            return;
        }

        const secondComponentResult = await Swal.fire({
            title: 'Enter second component of conjunction.',
            input: 'text',
            inputPlaceholder: 'B',
            showCloseButton: true,
        });

        let secondComponent = secondComponentResult.value;
        if (!secondComponent) {
            return;
        }

        secondComponent = this.parseProp(secondComponent);

        const conjunction: Prop = { kind: 'And', value: [conclusion.value, secondComponent] };

        return {
            additionalAssumptions: [],
            removedReasoingContextIds: [],
            newReasoningContexts: [],
            proofTreeChanges: [{
                newProofTree: {
                    ...proofTree,
                    rule: { kind: 'AndElimFst' },
                    premisses: [createEmptyVisualProofEditorProofTreeFromProp(conjunction)],
                },
                reasoningContextId,
                nodeId: proofTree.id,
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
            error('Conclusion is not a conjunction');
            return;
        }

        const propConclusion = conclusion.value;

        if (propConclusion.kind != 'And') {
            error('Conclusion is not a conjunction');
            return;
        }

        const [fst, _snd] = propConclusion.value;

        return {
            additionalAssumptions: [],
            removedReasoingContextIds: [],
            newReasoningContexts: [],
            proofTreeChanges: [{
                newProofTree: {
                    id: v4(),
                    premisses: [proofTree],
                    rule: { kind: 'AndElimFst' },
                    conclusion: { kind: 'PropIsTrue', value: fst },
                },
                nodeId: proofTree.id,
                reasoningContextId,
            }],
        };
    }
}
