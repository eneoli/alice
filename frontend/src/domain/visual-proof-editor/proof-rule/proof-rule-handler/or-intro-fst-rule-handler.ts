import Swal from 'sweetalert2';
import { VisualProofEditorRuleHandlerParams, ProofRuleHandlerResult } from '..';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../../../util/create-visual-proof-editor-empty-proof-tree';
import { ProofRuleHandler } from './proof-rule-handler';
import { v4 } from 'uuid';

export class OrIntroFstRuleHandler extends ProofRuleHandler {
    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const { proofTree } = params;
        const { id, conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            throw new Error('Conclusion is not a disjunction.');
        }

        const propConclusion = conclusion.value;

        if (propConclusion.kind !== 'Or') {
            throw new Error('Conclusion is not a disjunction.');
        }

        const [fst, _snd] = propConclusion.value;

        return {
            additionalAssumptions: [],
            newProofTree: {
                id,
                premisses: [createEmptyVisualProofEditorProofTreeFromProp(fst)],
                rule: { kind: 'OrIntroFst' },
                conclusion,
            },
        };
    }

    protected async handleRuleDownards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const { proofTree } = params;
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            throw new Error('Cannot apply rule on this node.');
        }

        const secondComponentResult = await Swal.fire({
            title: 'Enter second component of disjunction.',
            input: 'text',
            inputPlaceholder: 'B',
            showCloseButton: true,
        });

        let secondComponent = secondComponentResult.value;

        if (!secondComponent) {
            return this.createEmptyProofRuleHandlerResult();
        }

        secondComponent = this.parseProp(secondComponent);

        return {
            additionalAssumptions: [],
            newProofTree: {
                id: v4(),
                premisses: [proofTree],
                rule: { kind: 'OrIntroFst' },
                conclusion: { kind: 'PropIsTrue', value: { kind: 'Or', value: [conclusion.value, secondComponent] } },
            }
        };

    }

}
