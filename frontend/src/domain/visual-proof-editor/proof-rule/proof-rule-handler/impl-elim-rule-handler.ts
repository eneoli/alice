import { v4 } from 'uuid';
import { VisualProofEditorRuleHandlerParams, ProofRuleHandlerResult } from '..';
import { ProofRuleHandler } from './proof-rule-handler';
import Swal from 'sweetalert2';
import { isEqual } from 'lodash';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../lib/visual-proof-editor-proof-tree';

export class ImplElimRuleHandler extends ProofRuleHandler {

    public getLatexCode(): string {
        return `
            \\begin{prooftree}
                \\AxiomC{$A \\supset B$}
                \\AxiomC{$A$}
                \\RightLabel{$\\supset E$}
                \\BinaryInfC{$B$}
            \\end{prooftree}
        `;
    }

    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined> {
        const { selectedProofTreeNodes, error } = params;

        if (selectedProofTreeNodes.length !== 1) {
            error('Cannot apply rule on this node.');
            return;
        }

        const { proofTree, reasoningContextId } = selectedProofTreeNodes[0];
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            error('Cannot apply rule on this node.');
            return;
        }

        let implAntecedent = (await Swal.fire({
            title: 'Enter antecedent of implication.',
            input: 'text',
            inputPlaceholder: 'A',
            showCloseButton: true,
        })).value;

        if (!implAntecedent) {
            return;
        }

        implAntecedent = this.parseProp(implAntecedent);

        return {
            additionalAssumptions: [],
            removedReasoingContextIds: [],
            newReasoningContexts: [],
            proofTreeChanges: [{
                newProofTree: {
                    ...proofTree,
                    premisses: [
                        createEmptyVisualProofEditorProofTreeFromProp({ kind: 'Impl', value: [implAntecedent, conclusion.value] }),
                        createEmptyVisualProofEditorProofTreeFromProp(implAntecedent),
                    ],
                    rule: { kind: 'ImplElim' },
                },
                reasoningContextId,
                nodeId: proofTree.id,
            }],
        };
    }

    protected async handleRuleDownards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined> {
        const selectedNodesLength = params.selectedProofTreeNodes.length;
        const error = params.error;

        if (selectedNodesLength === 1) {
            return this.handleRuleDownwardsPrincipalConnectiveSelected(params);
        }

        if (selectedNodesLength === 2) {
            return this.handleRuleDownwardsBothPremissesSelected(params);
        }

        error('Cannot apply rule on this node.');
    }

    private async handleRuleDownwardsPrincipalConnectiveSelected(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined> {
        const { selectedProofTreeNodes, error } = params;

        if (selectedProofTreeNodes.length !== 1) {
            error('Expected exactly one selected node. This is likely a bug.');
            return;
        }

        const { proofTree, reasoningContextId } = selectedProofTreeNodes[0];
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            error('Conclusion is not an implication');
            return;
        }

        const propConclusion = conclusion.value;

        if (propConclusion.kind !== 'Impl') {
            error('Conclusion is not an implication');
            return;
        }

        const [fst, snd] = propConclusion.value;

        return {
            additionalAssumptions: [],
            newReasoningContexts: [],
            removedReasoingContextIds: [],
            proofTreeChanges: [{
                newProofTree: {
                    id: v4(),
                    premisses: [{ ...proofTree }, {
                        id: v4(),
                        premisses: [],
                        rule: null,
                        conclusion: { kind: 'PropIsTrue', value: fst },
                    }],
                    rule: { kind: 'ImplElim' },
                    conclusion: { kind: 'PropIsTrue', value: snd },
                },
                nodeId: proofTree.id,
                reasoningContextId,
            }],
        };
    }

    private async handleRuleDownwardsBothPremissesSelected(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined> {
        const { selectedProofTreeNodes, error } = params;

        if (selectedProofTreeNodes.length !== 2) {
            error('Expected exactly two selected nodes. This is likely a bug.');
            return;
        }

        const [fst, snd] = selectedProofTreeNodes;

        const fstConclusion = fst.proofTree.conclusion;
        const sndConclusion = snd.proofTree.conclusion;

        const fstIsProp = fstConclusion.kind === 'PropIsTrue';
        const sndIsProp = sndConclusion.kind === 'PropIsTrue';

        if (!fstIsProp || !sndIsProp) {
            error('Both premisses have to be proposition judgments.');
            return;
        }

        const fstIsImpl = fstConclusion.value.kind === 'Impl';
        const sndIsImpl = sndConclusion.value.kind === 'Impl';

        if (!fstIsImpl && !sndIsImpl) {
            throw new Error('Expected principal connective to be implication.');
        }

        // find principal connective

        let principal;
        let applicant;
        if (fstIsImpl && !sndIsImpl) {
            principal = fst;
            applicant = snd;
        }

        if (!fstIsImpl && sndIsImpl) {
            principal = snd;
            applicant = fst;
        }

        // We cannot use fstIsImpl and sndIsImpl because of Typescript's type system ._.
        if (fstConclusion.value.kind === 'Impl' && sndConclusion.value.kind === 'Impl') {
            const fstAntecedent = fstConclusion.value.value[0];

            if (fstAntecedent === sndConclusion.value) {
                principal = fst;
                applicant = snd;
            } else {
                principal = snd;
                applicant = fst;
            }
        }

        // check for compatibility

        // The first three checks are needed because TypeScript is not smart enough ._.

        if (
            principal?.proofTree.conclusion.kind !== 'PropIsTrue' ||
            applicant?.proofTree.conclusion.kind !== 'PropIsTrue' ||
            principal?.proofTree.conclusion.value.kind !== 'Impl' ||
            !isEqual(principal?.proofTree.conclusion.value.value![0], applicant?.proofTree.conclusion.value)
        ) {
            error('Premisses are not compatible.');
            return;
        }

        return {
            additionalAssumptions: [],
            proofTreeChanges: [],
            removedReasoingContextIds: [fst.reasoningContextId, snd.reasoningContextId],
            newReasoningContexts: [{
                id: v4(),
                isDragging: false,
                selectedNodeId: null,
                x: 0,
                y: 0,
                proofTree: {
                    id: v4(),
                    premisses: [fst.proofTree, snd.proofTree],
                    rule: { kind: 'ImplElim' },
                    conclusion: { kind: 'PropIsTrue', value: principal.proofTree.conclusion.value.value[1] }
                },
            }],
        };
    }
}