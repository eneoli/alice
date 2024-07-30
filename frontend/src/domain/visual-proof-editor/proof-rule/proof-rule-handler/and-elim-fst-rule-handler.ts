import { v4 } from 'uuid';
import { ProofRuleHandlerResult, VisualProofEditorRuleHandlerParams } from '..';
import Swal from 'sweetalert2';
import { ProofRuleHandler } from './proof-rule-handler';
import { Prop } from 'alice';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../../../util/create-visual-proof-editor-empty-proof-tree';

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

        const secondComponentResult = await Swal.fire({
            title: 'Enter second component of conjunction.',
            input: 'text',
            inputPlaceholder: 'B',
            showCloseButton: true,
        });

        let secondComponent = secondComponentResult.value;
        if (!secondComponent) {
            return this.createEmptyProofRuleHandlerResult();
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
