import { instantiate_free_parameter } from 'alice';
import { ProofRuleHandlerResult, VisualProofEditorRuleHandlerParams } from '..';
import { ProofRuleHandler } from './proof-rule-handler';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../lib/visual-proof-editor-proof-tree';

export class ForallIntroRuleHandler extends ProofRuleHandler {

    public getLatexCode(): string {
        return `
            \\begin{prooftree}
                \\AxiomC{$A(a)$}
                \\RightLabel{$\\forall I^{a}$}
                \\UnaryInfC{$\\forall x:\\tau. A(x)$}
            \\end{prooftree}
        `;
    }

    public willReasonDownwards(_params: VisualProofEditorRuleHandlerParams): boolean {
        return false;
    }

    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const {
            selectedProofTreeNodes,
            generateIdentifier,
            generateUniqueNumber,
        } = params;

        if (selectedProofTreeNodes.length !== 1) {
            throw new Error('Cannot apply this rule on multiple nodes.');
        }

        const { proofTree, reasoningContextId } = selectedProofTreeNodes[0];
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            throw new Error('Conclusion is not universal quantified');
        }

        const propConclusion = conclusion.value;

        if (propConclusion.kind !== 'ForAll') {
            throw new Error('Conclusion is not universal quantified');
        }

        const { object_ident, object_type_ident, body } = propConclusion.value;
        const paramIdent = {
            name: generateIdentifier(),
            unique_id: generateUniqueNumber(),
        };

        const intantiated_body = instantiate_free_parameter(body, object_ident, paramIdent);

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
                    rule: { kind: 'ForAllIntro', value: paramIdent.name },
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