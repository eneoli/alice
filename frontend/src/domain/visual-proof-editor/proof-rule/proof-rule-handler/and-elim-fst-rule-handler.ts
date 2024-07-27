import { v4 } from 'uuid';
import { ProofRuleHandlerResult, VisualProofEditorRuleHandlerParams } from '..';
import Swal from 'sweetalert2';
import { ProofRuleHandler } from './proof-rule-handler';
import { Prop } from 'alice';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../../../util/create-visual-proof-editor-empty-proof-tree';

export class AndElimFstRuleHandler extends ProofRuleHandler {
    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const { proofTree } = params;
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            throw new Error('Cannot apply rule on this node.');
        }

        const secondComponentResult = await Swal.fire({
            title: 'Enter second component of conjunction.',
            input: 'text',
            inputPlaceholder: 'B',
            showCloseButton: true,
        });

        let secondComponent = secondComponentResult.value;
        if (!secondComponent) {
            return this.createEmptyProofRuleHandlerResult();
        }

        secondComponent = this.parseProp(secondComponent);

        const conjunction: Prop = { kind: 'And', value: [conclusion.value, secondComponent] };

        return {
            additionalAssumptions: [],
            newProofTree: {
                ...proofTree,
                rule: { kind: 'AndElimFst' },
                premisses: [createEmptyVisualProofEditorProofTreeFromProp(conjunction)],
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

        const [fst, _snd] = propConclusion.value;

        return {
            additionalAssumptions: [],
            newProofTree: {
                id: v4(),
                premisses: [proofTree],
                rule: { kind: 'AndElimFst' },
                conclusion: { kind: 'PropIsTrue', value: fst },
            }
        };
    }
}
