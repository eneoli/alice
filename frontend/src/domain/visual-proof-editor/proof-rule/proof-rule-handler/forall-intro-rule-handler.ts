import { instantiate_free_parameter } from 'alice';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../../../util/create-visual-proof-editor-empty-proof-tree';
import { ProofRuleHandlerResult, VisualProofEditorRuleHandlerParams } from '..';
import { ProofRuleHandler } from './proof-rule-handler';

export class ForallIntroRuleHandler extends ProofRuleHandler {

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
            throw new Error('Conclusion is not universal quantified');
        }

        const propConclusion = conclusion.value;

        if (propConclusion.kind !== 'ForAll') {
            throw new Error('Conclusion is not universal quantified');
        }

        const { object_ident, object_type_ident, body } = propConclusion.value;
        const paramIdent = generateIdentifier();

        const intantiated_body = instantiate_free_parameter(body, object_ident, { name: paramIdent, unique_id: 0 });

        return {
            newReasoningContexts: [],
            removedReasoingContextIds: [],
            additionalAssumptions: [
                {
                    assumption: { kind: 'Datatype', ident: paramIdent, datatype: object_type_ident }, owningReasoningCtxId: reasoningContextId, owningNodeId: proofTree.id
                }
            ],
            proofTreeChanges: [{
                newProofTree: {
                    ...proofTree,
                    premisses: [createEmptyVisualProofEditorProofTreeFromProp(intantiated_body)],
                    rule: { kind: 'ForAllIntro', value: paramIdent },
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