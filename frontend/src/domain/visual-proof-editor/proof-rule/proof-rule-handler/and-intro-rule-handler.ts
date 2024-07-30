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

    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const { selectedProofTreeNodes } = params;

        if (selectedProofTreeNodes.length !== 1) {
            throw new Error('Cannot apply this rule on multiple nodes.');
        }

        const { proofTree, reasoningContextId } = selectedProofTreeNodes[0];
        const { rule, conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            throw new Error('Conclusion is not a conjunction');
        }

        const propConclusion = conclusion.value;

        if (propConclusion.kind !== 'And') {
            throw new Error('Conclusion is not a conjunction');
        }

        if (rule !== null) {
            throw new Error('Cannot reason upwards.');
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

    protected async handleRuleDownards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const { selectedProofTreeNodes } = params;

        if (selectedProofTreeNodes.length != 2) {
            throw new Error('Need exactly two nodes to combine them to a conjunction.');
        }

        const [fst, snd] = selectedProofTreeNodes;
        const fstConclusionKind = fst.proofTree.conclusion.kind;
        const sndConclusionKind = snd.proofTree.conclusion.kind;

        if (fstConclusionKind !== 'PropIsTrue' || sndConclusionKind !== 'PropIsTrue') {
            throw new Error('Cannot combine datatype to conjunction.');
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