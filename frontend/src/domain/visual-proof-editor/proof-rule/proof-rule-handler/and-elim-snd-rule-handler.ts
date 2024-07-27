import { v4 } from 'uuid';
import { ProofRuleHandlerResult, VisualProofEditorRuleHandlerParams } from '..';
import Swal from 'sweetalert2';
import { ProofRuleHandler } from './proof-rule-handler';
import { Prop } from 'alice';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../../../util/create-visual-proof-editor-empty-proof-tree';

export class AndElimSndRuleHandler extends ProofRuleHandler {
    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const { proofTree } = params;
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            throw new Error('Cannot apply rule on this node.');
        }

        const firstComponentResult = await Swal.fire({
            title: 'Enter first component of conjunction.',
            input: 'text',
            inputPlaceholder: 'A',
            showCloseButton: true,
        });

        let firstConclusion = firstComponentResult.value;
        if (!firstConclusion) {
            return this.createEmptyProofRuleHandlerResult();
        }

        firstConclusion = this.parseProp(firstConclusion);

        const conjunction: Prop = { kind: 'And', value: [firstConclusion, conclusion.value] };

        return {
            additionalAssumptions: [],
            newProofTree: {
                ...proofTree,
                premisses: [createEmptyVisualProofEditorProofTreeFromProp(conjunction)],
                rule: { kind: 'AndElimSnd' },
            },
        };

    }

    protected async handleRuleDownards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const { proofTree } = params;
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            throw new Error('Conclusion is not a conjunction');
        }

        const propConclusion = conclusion.value;

        if (propConclusion.kind != 'And') {
            throw new Error('Conclusion is not a conjunction');
        }

        const [_fst, snd] = propConclusion.value;

        return {
            additionalAssumptions: [],
            newProofTree: {
                id: v4(),
                premisses: [proofTree],
                rule: { kind: 'AndElimSnd' },
                conclusion: { kind: 'PropIsTrue', value: snd },
            }
        };
    }
}
