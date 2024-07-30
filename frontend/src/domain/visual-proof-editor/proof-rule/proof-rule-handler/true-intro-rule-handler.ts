import { VisualProofEditorRuleHandlerParams, ProofRuleHandlerResult } from '..';
import { ProofRuleHandler } from './proof-rule-handler';

export class TrueIntroRuleHandler extends ProofRuleHandler {

    public getLatexCode(): string {
        return `
            \\begin{prooftree}
                \\AxiomC{}
                \\RightLabel{$\\top I$}
                \\UnaryInfC{$\\top$}
            \\end{prooftree}
        `;
    }

    public willReasonDownwards(_params: VisualProofEditorRuleHandlerParams): boolean {
        return false;
    }

    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const { selectedProofTreeNodes } = params;

        if (selectedProofTreeNodes.length !== 1) {
            throw new Error('Cannot apply this rule on multiple nodes.');
        }

        const { proofTree, reasoningContextId } = selectedProofTreeNodes[0];
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            throw new Error('Conclusion is not truth.');
        }

        const propConclusion = conclusion.value;

        if (propConclusion.kind !== 'True') {
            throw new Error('Conclusion is not truth.');
        }

        return {
            additionalAssumptions: [],
            removedReasoingContextIds: [],
            newReasoningContexts: [],
            proofTreeChanges: [{
                newProofTree: {
                    ...proofTree,
                    rule: { kind: 'TrueIntro' },
                },
                nodeId: proofTree.id,
                reasoningContextId,
            }]
        };
    }

    protected handleRuleDownards(_params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        throw new Error('Cannot reason downwards with this rule.');
    }
}