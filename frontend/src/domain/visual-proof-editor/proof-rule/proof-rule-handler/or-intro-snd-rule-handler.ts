import Swal from 'sweetalert2';
import { VisualProofEditorRuleHandlerParams, ProofRuleHandlerResult } from '..';
import { ProofRuleHandler } from './proof-rule-handler';
import { v4 } from 'uuid';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../lib/visual-proof-editor-proof-tree';

export class OrIntroSndRuleHandler extends ProofRuleHandler {

    public getLatexCode(): string {
        return `
            \\begin{prooftree}
                \\AxiomC{B}
                \\RightLabel{$\\lor I_2$}
                \\UnaryInfC{$A \\lor B$}
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
        const { id, conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            error('Conclusion is not a disjunction.');
            return;
        }

        const propConclusion = conclusion.value;

        if (propConclusion.kind !== 'Or') {
            error('Conclusion is not a disjunction.');
            return;
        }

        const [_fst, snd] = propConclusion.value;

        return {
            additionalAssumptions: [],
            newReasoningContexts: [],
            removedReasoingContextIds: [],
            proofTreeChanges: [{
                newProofTree: {
                    id: id,
                    premisses: [createEmptyVisualProofEditorProofTreeFromProp(snd)],
                    rule: { kind: 'OrIntroSnd' },
                    conclusion,
                },
                nodeId: proofTree.id,
                reasoningContextId,
            }],
        };
    }

    protected async handleRuleDownards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined> {
        const { selectedProofTreeNodes, error } = params;

        if (selectedProofTreeNodes.length !== 1) {
            error('Cannot apply rule on this node.');
            return;
        }

        const { proofTree, reasoningContextId } = selectedProofTreeNodes[0];
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            error('Cannot apply rule on this node.');
            return;
        }

        const firstComponentResult = await Swal.fire({
            title: 'Enter first component of disjunction.',
            input: 'text',
            inputPlaceholder: 'A',
            showCloseButton: true,
        });

        let firstComponent = firstComponentResult.value;

        if (!firstComponent) {
            return;
        }

        firstComponent = this.parseProp(firstComponent);

        return {
            additionalAssumptions: [],
            newReasoningContexts: [],
            removedReasoingContextIds: [],
            proofTreeChanges: [{
                newProofTree: {
                    id: v4(),
                    premisses: [proofTree],
                    rule: { kind: 'OrIntroSnd' },
                    conclusion: { kind: 'PropIsTrue', value: { kind: 'Or', value: [firstComponent, conclusion.value] } },
                },
                nodeId: proofTree.id,
                reasoningContextId,
            }]
        };
    }
}
