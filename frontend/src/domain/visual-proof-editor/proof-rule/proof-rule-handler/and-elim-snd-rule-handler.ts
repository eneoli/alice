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

    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const { selectedProofTreeNodes: selecteedProofTreeNodes } = params;

        if (selecteedProofTreeNodes.length !== 1) {
            throw new Error('Cannot apply this rule on multiple nodes.');
        }

        const { proofTree, reasoningContextId } = selecteedProofTreeNodes[0];
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            throw new Error('Cannot apply rule on this node.');
        }

        const firstComponentResult = await Swal.fire({
            title: 'Enter first component of conjunction.',
            input: 'text',
            inputPlaceholder: 'A',
            showCloseButton: true,
        });

        let firstConclusion = firstComponentResult.value;
        if (!firstConclusion) {
            return this.createEmptyProofRuleHandlerResult();
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

    protected async handleRuleDownards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const { selectedProofTreeNodes: selecteedProofTreeNodes } = params;

        if (selecteedProofTreeNodes.length !== 1) {
            throw new Error('Cannot apply this rule on multiple nodes.');
        }

        const { proofTree, reasoningContextId } = selecteedProofTreeNodes[0];
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            throw new Error('Conclusion is not a conjunction');
        }

        const propConclusion = conclusion.value;

        if (propConclusion.kind != 'And') {
            throw new Error('Conclusion is not a conjunction');
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
