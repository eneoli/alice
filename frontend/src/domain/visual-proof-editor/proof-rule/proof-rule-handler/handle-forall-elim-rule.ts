import { v4 } from 'uuid';
import Swal from 'sweetalert2';
import { instantiate_free_parameter } from 'alice';
import { VisualProofEditorRuleHandlerParams, ProofRuleHandlerResult } from '..';

export async function handleForAllElimRule({ proofTree }: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
    const { conclusion } = proofTree;

    if (conclusion.kind !== 'PropIsTrue') {
        throw new Error('Conclusion is not an universal quantification');
    }

    const propConclusion = conclusion.value;

    if (propConclusion.kind !== 'ForAll') {
        throw new Error('Conclusion is not an universal quantification');
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
            id: v4(),
            premisses: [{ ...proofTree }, {
                id: v4(),
                premisses: [],
                rule: null,
                conclusion: { kind: 'TypeJudgement', value: [instantiationIdentifier.value, object_type_ident] },
            }],
            rule: { kind: 'ForAllElim' },
            conclusion: { kind: 'PropIsTrue', value: instantiated_body },
        }
    };
}