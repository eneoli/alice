import { VisualProofEditorRuleHandlerParams, ProofRuleHandlerResult } from '..';
import { ProofRuleHandler } from './proof-rule-handler';

export class TrueIntroRuleHandler extends ProofRuleHandler {

    public willReasonDownwards(_params: VisualProofEditorRuleHandlerParams): boolean {
        return false;
    }

    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const { proofTree } = params;
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
            newProofTree: {
                ...proofTree,
                rule: { kind: 'TrueIntro' },
            },
        };
    }

    protected handleRuleDownards(_params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        throw new Error('Cannot reason downwards with this rule.');
    }
}