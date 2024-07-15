import Swal from 'sweetalert2';
import { instantiate_free_parameter, parse_prop } from 'alice';
import { v4 } from 'uuid';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../../../util/create-visual-proof-editor-empty-proof-tree';
import { VisualProofEditorRuleHandlerParams, ProofRuleHandlerResult } from '..';

export async function handleExistsElimRule({ proofTree, reasoningContextId, generateIdentifier }: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
    const { conclusion } = proofTree;

    if (conclusion.kind !== 'PropIsTrue') {
        throw new Error('Conclusion is not an existential quantification');
    }

    const propConclusion = conclusion.value;

    if (propConclusion.kind !== 'Exists') {
        throw new Error('Conclusion is not an existential quantification');
    }

    // ask for new conclusion
    const newConclusionPromptResult = await Swal.fire({
        title: 'Enter new conclusion',
        input: 'text',
        inputLabel: 'Conclusion',
        inputPlaceholder: 'C',
        showCloseButton: true,
    });

    if (!newConclusionPromptResult.isConfirmed) {
        return {
            additionalAssumptions: [],
            newProofTree: proofTree,
        };
    }

    const newConclusion = parse_prop(newConclusionPromptResult.value);

    const { object_ident, object_type_ident, body } = propConclusion.value;

    const objectIdent = generateIdentifier();
    const propIdent = generateIdentifier();

    const instantiated_body = instantiate_free_parameter(body, object_ident, { name: objectIdent, unique_id: 0 });

    const nodeId = v4();

    return {
        additionalAssumptions: [
            { assumption: { kind: 'Datatype', ident: objectIdent, datatype: object_type_ident }, owningReasoningCtxId: reasoningContextId, owningNodeId: nodeId },
            { assumption: { kind: 'PropIsTrue', ident: propIdent, prop: instantiated_body }, owningReasoningCtxId: reasoningContextId, owningNodeId: nodeId }
        ],
        newProofTree: {
            id: nodeId,
            premisses: [
                { ...proofTree },
                createEmptyVisualProofEditorProofTreeFromProp(newConclusion),
            ],
            rule: { kind: 'ExistsElim', value: [objectIdent, propIdent] },
            conclusion: { kind: 'PropIsTrue', value: newConclusion },
        }
    };
}