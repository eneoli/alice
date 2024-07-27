import { v4 } from 'uuid';
import { VisualProofEditorRuleHandlerParams, ProofRuleHandlerResult } from '..';
import { ProofRuleHandler } from './proof-rule-handler';
import Swal from 'sweetalert2';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../../../util/create-visual-proof-editor-empty-proof-tree';

export class ImplElimRuleHandler extends ProofRuleHandler {
    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const { proofTree } = params;
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            throw new Error('Cannot apply rule on this node.');
        }

        let implAntecedent = (await Swal.fire({
            title: 'Enter antecedent of implication.',
            input: 'text',
            inputPlaceholder: 'A',
            showCloseButton: true,
        })).value;

        if (!implAntecedent) {
            return this.createEmptyProofRuleHandlerResult();
        }

        implAntecedent = this.parseProp(implAntecedent);

        return {
            additionalAssumptions: [],
            newProofTree: {
                ...proofTree,
                premisses: [
                    createEmptyVisualProofEditorProofTreeFromProp({ kind: 'Impl', value: [implAntecedent, conclusion.value] }),
                    createEmptyVisualProofEditorProofTreeFromProp(implAntecedent),
                ],
                rule: { kind: 'ImplElim' },
            },
        };
    }

    protected async handleRuleDownards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const { proofTree } = params;
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            throw new Error('Conclusion is not an implication');
        }

        const propConclusion = conclusion.value;

        if (propConclusion.kind !== 'Impl') {
            throw new Error('Conclusion is not an implication');
        }

        const [fst, snd] = propConclusion.value;

        return {
            additionalAssumptions: [],
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
            }
        };
    }
}
