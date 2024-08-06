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

    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined> {
        const { selectedProofTreeNodes, error } = params;

        if (selectedProofTreeNodes.length !== 1) {
            error('Cannot apply this rule on multiple nodes.');
            return;
        }

        const { proofTree, reasoningContextId } = selectedProofTreeNodes[0];
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            error('Conclusion is not truth.');
            return;
        }

        const propConclusion = conclusion.value;

        if (propConclusion.kind !== 'True') {
            error('Conclusion is not truth.');
            return;
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

    protected async handleRuleDownards({ error }: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined> {
        error('Cannot reason downwards with this rule.');
        return;
    }
}