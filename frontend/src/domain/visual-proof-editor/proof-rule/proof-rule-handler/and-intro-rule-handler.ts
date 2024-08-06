import { v4 } from 'uuid';
import { ProofRuleHandlerResult, VisualProofEditorRuleHandlerParams } from '..';
import { ProofRuleHandler } from './proof-rule-handler';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../lib/visual-proof-editor-proof-tree';

export class AndIntroRuleHandler extends ProofRuleHandler {

    public getLatexCode(): string {
        return `
            \\begin{prooftree}
                \\AxiomC{$A$}
                \\AxiomC{$B$}
                \\RightLabel{$\\land I$}
                \\BinaryInfC{$A \\land B$}
            \\end{prooftree}
        `;
    }

    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined> {
        const { selectedProofTreeNodes, error: fail } = params;

        if (selectedProofTreeNodes.length !== 1) {
            fail('Cannot apply this rule on multiple nodes.');
            return;
        }

        const { proofTree, reasoningContextId } = selectedProofTreeNodes[0];
        const { rule, conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            fail('Conclusion is not a conjunction');
            return;
        }

        const propConclusion = conclusion.value;

        if (propConclusion.kind !== 'And') {
            fail('Conclusion is not a conjunction');
            return;
        }

        if (rule !== null) {
            fail('Cannot reason upwards.');
            return;
        }

        const [fst, snd] = propConclusion.value;

        return {
            additionalAssumptions: [],
            removedReasoingContextIds: [],
            newReasoningContexts: [],
            proofTreeChanges: [{
                newProofTree: {
                    ...proofTree,
                    premisses: [
                        createEmptyVisualProofEditorProofTreeFromProp(fst),
                        createEmptyVisualProofEditorProofTreeFromProp(snd),
                    ],
                    rule: { kind: 'AndIntro' },
                },
                nodeId: proofTree.id,
                reasoningContextId,
            }],
        };
    }

    protected async handleRuleDownards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined> {
        const { selectedProofTreeNodes, error } = params;

        if (selectedProofTreeNodes.length != 2) {
            error('Need exactly two nodes to combine them to a conjunction.');
            return;
        }

        const [fst, snd] = selectedProofTreeNodes;
        const fstConclusionKind = fst.proofTree.conclusion.kind;
        const sndConclusionKind = snd.proofTree.conclusion.kind;

        if (fstConclusionKind !== 'PropIsTrue' || sndConclusionKind !== 'PropIsTrue') {
            error('Cannot combine datatype to conjunction.');
            return;
        }

        const fstProp = fst.proofTree.conclusion.value;
        const sndProp = snd.proofTree.conclusion.value;

        return {
            additionalAssumptions: [],
            proofTreeChanges: [],
            removedReasoingContextIds: [fst.reasoningContextId, snd.reasoningContextId],
            newReasoningContexts: [{
                id: v4(),
                selectedNodeId: null,
                isDragging: false,
                x: 0,
                y: 0,
                proofTree: {
                    id: v4(),
                    premisses: [fst.proofTree, snd.proofTree],
                    rule: { kind: 'AndIntro' },
                    conclusion: { kind: 'PropIsTrue', value: { kind: 'And', value: [fstProp, sndProp] } }
                },
            }],
        };
    }
}