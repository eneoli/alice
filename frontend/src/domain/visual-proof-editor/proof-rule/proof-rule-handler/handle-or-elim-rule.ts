import Swal from 'sweetalert2';
import { VisualProofEditorProofTree } from '../../components/visual-proof-editor';
import { ProofRuleHandlerResult } from '../../components/visual-proof-editor-sidebar';
import { parse_prop } from 'alice';
import { v4 } from 'uuid';
import { createEmptyVisualProofEditorProofTree } from '../../../../util/create-visual-proof-editor-empty-proof-tree';
import { generateIdentifier } from './generate-identifier';

export async function handleOrElimRule(proofTree: VisualProofEditorProofTree): Promise<ProofRuleHandlerResult> {
    const { conclusion } = proofTree;

    if (conclusion.kind !== 'Or') {
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

    const [fst, snd] = conclusion.value;

    const newConclusion = parse_prop(newConclusionPromptResult.value);

    const fstIdent = generateIdentifier();
    const sndIdent = generateIdentifier();

    return {
        additionalAssumptions: [
            { kind: 'PropIsTrue', ident: fstIdent, prop: fst },
            { kind: 'PropIsTrue', ident: sndIdent, prop: snd }
        ],
        newProofTree: {
            id: v4(),
            premisses: [
                { ...proofTree },
                createEmptyVisualProofEditorProofTree(newConclusion),
                createEmptyVisualProofEditorProofTree(newConclusion),
            ],
            rule: { kind: 'OrElim', value: [fstIdent, sndIdent] },
            conclusion: newConclusion,
        }
    };
}