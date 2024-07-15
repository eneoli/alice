import Swal from 'sweetalert2';
import { parse_prop } from 'alice';
import { v4 } from 'uuid';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../../../util/create-visual-proof-editor-empty-proof-tree';
import { VisualProofEditorRuleHandlerParams, ProofRuleHandlerResult } from '..';

export async function handleOrElimRule({ proofTree, reasoningContextId, generateIdentifier }: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
    const { conclusion } = proofTree;

    if (conclusion.kind !== 'PropIsTrue') {
        throw new Error('Conclusion is not an implication');
    }

    const propConclusion = conclusion.value;

    if (propConclusion.kind !== 'Or') {
        throw new Error('Conclusion is not an implication');
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

    const [fst, snd] = propConclusion.value;

    const newConclusion = parse_prop(newConclusionPromptResult.value);

    const fstIdent = generateIdentifier();
    const sndIdent = generateIdentifier();

    const nodeId = v4();

    return {
        additionalAssumptions: [
            { assumption: { kind: 'PropIsTrue', ident: fstIdent, prop: fst }, owningReasoningCtxId: reasoningContextId, owningNodeId: nodeId },
            { assumption: { kind: 'PropIsTrue', ident: sndIdent, prop: snd }, owningReasoningCtxId: reasoningContextId, owningNodeId: nodeId },
        ],
        newProofTree: {
            id: nodeId,
            premisses: [
                { ...proofTree },
                createEmptyVisualProofEditorProofTreeFromProp(newConclusion),
                createEmptyVisualProofEditorProofTreeFromProp(newConclusion),
            ],
            rule: { kind: 'OrElim', value: [fstIdent, sndIdent] },
            conclusion: { kind: 'PropIsTrue', value: newConclusion },
        }
    };
}