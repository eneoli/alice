import { VisualProofEditorRuleHandlerParams, ProofRuleHandlerResult, SelectedProofTreeNode } from '..';
import { ProofRuleHandler } from './proof-rule-handler';
import { v4 } from 'uuid';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../lib/visual-proof-editor-proof-tree';

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

    public canReasonUpwards(nodes: SelectedProofTreeNode[]): boolean {
        return (
            super.canReasonUpwards(nodes) &&
            nodes.length === 1 &&
            nodes[0].proofTree.conclusion.kind === 'PropIsTrue' &&
            nodes[0].proofTree.conclusion.value.kind === 'Or'
        );
    }

    public canReasonDownwards(nodes: SelectedProofTreeNode[]): boolean {
        return (
            super.canReasonDownwards(nodes) &&
            nodes.length === 1 &&
            nodes[0].proofTree.conclusion.kind === 'PropIsTrue'
        );
    }

    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined> {
        const { selectedProofTreeNodes, error } = params;

        if (selectedProofTreeNodes.length !== 1) {
            error('Cannot apply rule on this node.');
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

    protected async handleRuleDownards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined> {
        const { selectedProofTreeNodes, assumptions, error } = params;

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

        // secondComponent = this.parseProp(secondComponent);
        const secondComponent = await this.promptProp({
            title: 'Enter second component of disjunction.',
            inputPlaceholder: 'B',
            assumptions,
            error,
        });

        if (!secondComponent) {
            return;
        }

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
