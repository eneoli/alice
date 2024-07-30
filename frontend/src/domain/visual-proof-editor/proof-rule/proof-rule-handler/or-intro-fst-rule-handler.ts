import Swal from 'sweetalert2';
import { VisualProofEditorRuleHandlerParams, ProofRuleHandlerResult } from '..';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../../../util/create-visual-proof-editor-empty-proof-tree';
import { ProofRuleHandler } from './proof-rule-handler';
import { v4 } from 'uuid';

export class OrIntroFstRuleHandler extends ProofRuleHandler {

    public getLatexCode(): string {
        return `
            \\begin{prooftree}
                \\AxiomC{A}
                \\RightLabel{$\\lor I_1$}
                \\UnaryInfC{$A \\lor B$}
            \\end{prooftree}
        `;
    }

    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const { selectedProofTreeNodes: selecteedProofTreeNodes } = params;

        if (selecteedProofTreeNodes.length !== 1) {
            throw new Error('Cannot apply rule on this node.');
        }

        const { proofTree, reasoningContextId } = selecteedProofTreeNodes[0];
        const { id, conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            throw new Error('Conclusion is not a disjunction.');
        }

        const propConclusion = conclusion.value;

        if (propConclusion.kind !== 'Or') {
            throw new Error('Conclusion is not a disjunction.');
        }

        const [fst, _snd] = propConclusion.value;

        return {
            additionalAssumptions: [],
            removedReasoingContextIds: [],
            newReasoningContexts: [],
            proofTreeChanges: [{
                newProofTree: {
                    id,
                    premisses: [createEmptyVisualProofEditorProofTreeFromProp(fst)],
                    rule: { kind: 'OrIntroFst' },
                    conclusion,
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
            throw new Error('Cannot apply rule on this node.');
        }

        const secondComponentResult = await Swal.fire({
            title: 'Enter second component of disjunction.',
            input: 'text',
            inputPlaceholder: 'B',
            showCloseButton: true,
        });

        let secondComponent = secondComponentResult.value;

        if (!secondComponent) {
            return this.createEmptyProofRuleHandlerResult();
        }

        secondComponent = this.parseProp(secondComponent);

        return {
            additionalAssumptions: [],
            newReasoningContexts: [],
            removedReasoingContextIds: [],
            proofTreeChanges: [{
                newProofTree: {
                    id: v4(),
                    premisses: [proofTree],
                    rule: { kind: 'OrIntroFst' },
                    conclusion: { kind: 'PropIsTrue', value: { kind: 'Or', value: [conclusion.value, secondComponent] } },
                },
                nodeId: proofTree.id,
                reasoningContextId,
            }]
        };
    }
}
