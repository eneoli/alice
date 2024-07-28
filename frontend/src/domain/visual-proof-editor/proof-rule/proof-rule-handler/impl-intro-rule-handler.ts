import { VisualProofEditorRuleHandlerParams, ProofRuleHandlerResult } from '..';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../../../util/create-visual-proof-editor-empty-proof-tree';
import { ProofRuleHandler } from './proof-rule-handler';

export class ImplIntroRuleHandler extends ProofRuleHandler {

    public willReasonDownwards(_params: VisualProofEditorRuleHandlerParams): boolean {
        return false;
    }

    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const { selectedProofTreeNodes: selecteedProofTreeNodes, generateIdentifier } = params;

        if (selecteedProofTreeNodes.length !== 1) {
            throw new Error('Cannot apply this rule on multiple nodes.');
        }

        const { proofTree, reasoningContextId } = selecteedProofTreeNodes[0];
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
            proofTreeChanges: [{
                newProofTree: {
                    id: proofTree.id,
                    premisses: [createEmptyVisualProofEditorProofTreeFromProp(snd)],
                    rule: { kind: 'ImplIntro', value: ident },
                    conclusion,
                },
                nodeId: proofTree.id,
                reasoningContextId,
            }],
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
            newReasoningContexts: [],
            removedReasoingContextIds: [],
        };
    }

    protected handleRuleDownards(_params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        throw new Error('Cannot reason downards with this rule.');
    }

}

