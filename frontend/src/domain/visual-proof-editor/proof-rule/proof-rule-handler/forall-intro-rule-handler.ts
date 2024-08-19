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

    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined> {
        const {
            selectedProofTreeNodes,
            generateIdentifier,
            generateUniqueNumber,
            error,
        } = params;

        if (selectedProofTreeNodes.length !== 1) {
            error('Cannot apply this rule on multiple nodes.');
            return;
        }

        const { proofTree, reasoningContextId } = selectedProofTreeNodes[0];
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            error('Conclusion is not universal quantified');
            return;
        }

        const propConclusion = conclusion.value;

        if (propConclusion.kind !== 'ForAll') {
            error('Conclusion is not universal quantified');
            return;
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
                    rule: { kind: 'ForAllIntro', value: paramIdent },
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