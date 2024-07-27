import Swal from 'sweetalert2';
import { VisualProofEditorRuleHandlerParams, ProofRuleHandlerResult } from '..';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../../../util/create-visual-proof-editor-empty-proof-tree';
import { ProofRuleHandler } from './proof-rule-handler';
import { v4 } from 'uuid';

export class OrIntroSndRuleHandler extends ProofRuleHandler {
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

        const [_fst, snd] = propConclusion.value;

        return {
            additionalAssumptions: [],
            newProofTree: {
                id: id,
                premisses: [createEmptyVisualProofEditorProofTreeFromProp(snd)],
                rule: { kind: 'OrIntroSnd' },
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

        const firstComponentResult = await Swal.fire({
            title: 'Enter first component of disjunction.',
            input: 'text',
            inputPlaceholder: 'A',
            showCloseButton: true,
        });

        let firstComponent = firstComponentResult.value;

        if (!firstComponent) {
            return this.createEmptyProofRuleHandlerResult();
        }

        firstComponent = this.parseProp(firstComponent);

        return {
            additionalAssumptions: [],
            newProofTree: {
                id: v4(),
                premisses: [proofTree],
                rule: { kind: 'OrIntroSnd' },
                conclusion: { kind: 'PropIsTrue', value: { kind: 'Or', value: [firstComponent, conclusion.value] } },
            }
        };
    }
}
