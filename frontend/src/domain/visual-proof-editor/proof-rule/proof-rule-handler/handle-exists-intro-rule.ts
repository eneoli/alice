import { instantiate_free_parameter } from 'alice';
import Swal from 'sweetalert2';
import { createEmptyVisualProofEditorProofTreeFromProp, createEmptyVisualProofEditorProofTreeFromTypeJudgment } from '../../../../util/create-visual-proof-editor-empty-proof-tree';
import { ProofRuleHandlerResult, VisualProofEditorRuleHandlerParams } from '..';

export async function handleExistsIntroRule({ proofTree }: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
    const { conclusion } = proofTree;

    if (conclusion.kind !== 'PropIsTrue') {
        throw new Error('Conclusion is not an existential quantification');
    }

    const propConclusion = conclusion.value;

    if (propConclusion.kind !== 'Exists') {
        throw new Error('Conclusion is not an existential quantification');
    }

    // ask for instantiation identifier
    const instantiationIdentifier = await Swal.fire({
        title: 'Enter instantiation identifier',
        input: 'text',
        inputLabel: 'Identifier',
        inputPlaceholder: 'a',
        showCloseButton: true,
    });

    const { object_ident, object_type_ident, body } = propConclusion.value;

    const instantiated_body = instantiate_free_parameter(body, object_ident, { name: instantiationIdentifier.value, unique_id: 0 });

    return {
        additionalAssumptions: [],
        newProofTree: {
            id: proofTree.id,
            premisses: [
                createEmptyVisualProofEditorProofTreeFromTypeJudgment(instantiationIdentifier.value, object_type_ident),
                createEmptyVisualProofEditorProofTreeFromProp(instantiated_body),
            ],
            rule: { kind: 'ExistsIntro' },
            conclusion,
        },
    };
}