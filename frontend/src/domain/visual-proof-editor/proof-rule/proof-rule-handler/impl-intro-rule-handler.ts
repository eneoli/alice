import { VisualProofEditorRuleHandlerParams, ProofRuleHandlerResult } from '..';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../../../util/create-visual-proof-editor-empty-proof-tree';
import { ProofRuleHandler } from './proof-rule-handler';

export class ImplIntroRuleHandler extends ProofRuleHandler {

    public willReasonDownwards(_params: VisualProofEditorRuleHandlerParams): boolean {
        return false;
    }

    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const { proofTree, reasoningContextId, generateIdentifier } = params;
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            throw new Error('Conclusion is not an implication.');
        }

        const propConclusion = conclusion.value;

        if (propConclusion.kind != 'Impl') {
            throw new Error('Conclusion is not an implication.');
        }

        const [fst, snd] = propConclusion.value;

        const ident = generateIdentifier();

        return {
            newProofTree: {
                id: proofTree.id,
                premisses: [createEmptyVisualProofEditorProofTreeFromProp(snd)],
                rule: { kind: 'ImplIntro', value: ident },
                conclusion,
            },
            additionalAssumptions: [
                {
                    assumption: {
                        kind: 'PropIsTrue',
                        prop: fst,
                        ident,
                    },
                    owningReasoningCtxId: reasoningContextId,
                    owningNodeId: proofTree.id,
                }
            ],
        };
    }

    protected handleRuleDownards(_params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        throw new Error('Cannot reason downards with this rule.');
    }

}

