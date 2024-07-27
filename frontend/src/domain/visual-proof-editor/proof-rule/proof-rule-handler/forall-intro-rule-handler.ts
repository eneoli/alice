import { instantiate_free_parameter } from 'alice';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../../../util/create-visual-proof-editor-empty-proof-tree';
import { ProofRuleHandlerResult, VisualProofEditorRuleHandlerParams } from '..';
import { ProofRuleHandler } from './proof-rule-handler';

export class ForallIntroRuleHandler extends ProofRuleHandler {

    public willReasonDownwards(_params: VisualProofEditorRuleHandlerParams): boolean {
        return false;
    }

    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const { proofTree, reasoningContextId, generateIdentifier } = params;
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
            additionalAssumptions: [
                {
                    assumption: { kind: 'Datatype', ident: paramIdent, datatype: object_type_ident }, owningReasoningCtxId: reasoningContextId, owningNodeId: proofTree.id
                }
            ],
            newProofTree: {
                ...proofTree,
                premisses: [createEmptyVisualProofEditorProofTreeFromProp(intantiated_body)],
                rule: { kind: 'ForAllIntro', value: paramIdent },
            },
        };
    }

    protected handleRuleDownards(_params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        throw new Error('Cannot reason downwards with this rule.');
    }

}