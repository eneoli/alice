import { Identifier } from 'alice';
import { VisualProofEditorRuleHandlerParams, ProofRuleHandlerResult } from '..';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../lib/visual-proof-editor-proof-tree';
import { ProofRuleHandler } from './proof-rule-handler';

export class ImplIntroRuleHandler extends ProofRuleHandler {

    public getLatexCode(): string {
        return `
            \\begin{prooftree}
                \\AxiomC{[A]}
                \\noLine
                \\UnaryInfC{B}
                \\RightLabel{$\\supset I$}
                \\UnaryInfC{$A \\supset B$}
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
            throw new Error('Conclusion is not an implication.');
        }

        const propConclusion = conclusion.value;

        if (propConclusion.kind != 'Impl') {
            throw new Error('Conclusion is not an implication.');
        }

        const [fst, snd] = propConclusion.value;

        const ident: Identifier = {
            name: generateIdentifier(),
            unique_id: generateUniqueNumber(),
        };

        return {
            proofTreeChanges: [{
                newProofTree: {
                    ...proofTree,
                    premisses: [createEmptyVisualProofEditorProofTreeFromProp(snd)],
                    rule: { kind: 'ImplIntro', value: ident.name },
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

