import { Identifier } from 'alice';
import { VisualProofEditorRuleHandlerParams, ProofRuleHandlerResult, SelectedProofTreeNode } from '..';
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

    public canReasonDownwards(_nodes: SelectedProofTreeNode[]): boolean {
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
            error('Conclusion is not an implication.');
            return;
        }

        const propConclusion = conclusion.value;

        if (propConclusion.kind != 'Impl') {
            error('Conclusion is not an implication.');
            return;
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
                    rule: { kind: 'ImplIntro', value: ident },
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

    protected async handleRuleDownards({ error }: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined> {
        error('Cannot reason downards with this rule.');
        return;
    }

}

