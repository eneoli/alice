import { v4 } from 'uuid';
import { ProofRuleHandlerResult, VisualProofEditorRuleHandlerParams } from '..';
import Swal from 'sweetalert2';
import { ProofRuleHandler } from './proof-rule-handler';
import { Prop } from 'alice';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../lib/visual-proof-editor-proof-tree';

export class AndElimSndRuleHandler extends ProofRuleHandler {

    public getLatexCode(): string {
        return `
            \\begin{prooftree}
                \\AxiomC{$A \\land B$}
                \\RightLabel{$\\land E_2$}
                \\UnaryInfC{$B$}
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

        const firstComponentResult = await Swal.fire({
            title: 'Enter first component of conjunction.',
            input: 'text',
            inputPlaceholder: 'A',
            showCloseButton: true,
        });

        let firstConclusion = firstComponentResult.value;
        if (!firstConclusion) {
            return;
        }

        firstConclusion = this.parseProp(firstConclusion);

        const conjunction: Prop = { kind: 'And', value: [firstConclusion, conclusion.value] };

        return {
            additionalAssumptions: [],
            removedReasoingContextIds: [],
            newReasoningContexts: [],
            proofTreeChanges: [{
                newProofTree: {
                    ...proofTree,
                    premisses: [createEmptyVisualProofEditorProofTreeFromProp(conjunction)],
                    rule: { kind: 'AndElimSnd' },
                },
                reasoningContextId,
                nodeId: proofTree.id,
            }],
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

        const [_fst, snd] = propConclusion.value;

        return {
            additionalAssumptions: [],
            newReasoningContexts: [],
            removedReasoingContextIds: [],
            proofTreeChanges: [{
                newProofTree: {
                    id: v4(),
                    premisses: [proofTree],
                    rule: { kind: 'AndElimSnd' },
                    conclusion: { kind: 'PropIsTrue', value: snd },
                },
                nodeId: proofTree.id,
                reasoningContextId,
            }],
        };
    }
}
