import Swal from 'sweetalert2';
import { instantiate_free_parameter } from 'alice';
import { v4 } from 'uuid';
import { VisualProofEditorRuleHandlerParams, ProofRuleHandlerResult } from '..';
import { ProofRuleHandler } from './proof-rule-handler';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../lib/visual-proof-editor-proof-tree';

export class ExistsElimRuleHandler extends ProofRuleHandler {

    public getLatexCode(): string {
        return `
            \\begin{prooftree}
                \\AxiomC{$\\exists x: \\tau. A(x)$}
                \\AxiomC{[$A(a)$]}
                \\AxiomC{[$a: \\tau$]}
                \\noLine
                \\BinaryInfC{$C$}
                \\RightLabel{$\\exists E$}
                \\BinaryInfC{$C$}
            \\end{prooftree}
        `;
    }

    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const {
            selectedProofTreeNodes,
            generateIdentifier,
            generateUniqueNumber,
        } = params;

        if (selectedProofTreeNodes.length !== 1) {
            throw new Error('Cannot apply this rule on multiple nodes.');
        }

        const { proofTree, reasoningContextId } = selectedProofTreeNodes[0];
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            throw new Error('Cannot apply rule on this node.');
        }

        let existsProp = (await Swal.fire({
            title: 'Enter existential quantification you want to eliminate.',
            input: 'text',
            inputPlaceholder: '∃x:t. A(x)',
            showCloseButton: true,
        })).value;

        if (!existsProp) {
            return this.createEmptyProofRuleHandlerResult();
        }

        existsProp = this.parseProp(existsProp);

        if (existsProp.kind !== 'Exists') {
            throw new Error('You did not enter an existential quantification.');
        }

        const instantiatedObjectIdent = {
            name: generateIdentifier(),
            unique_id: generateUniqueNumber(),
        };

        const propIdent = {
            name: generateIdentifier(),
            unique_id: generateUniqueNumber(),
        };

        const { object_ident, object_type_ident, body } = existsProp.value;
        const instantiatedBody = instantiate_free_parameter(body, object_ident, instantiatedObjectIdent);

        return {
            newReasoningContexts: [],
            removedReasoingContextIds: [],
            additionalAssumptions: [
                { assumption: { kind: 'Datatype', ident: instantiatedObjectIdent, datatype: object_type_ident }, owningReasoningCtxId: reasoningContextId, owningNodeId: proofTree.id },
                { assumption: { kind: 'PropIsTrue', ident: propIdent, prop: instantiatedBody }, owningReasoningCtxId: reasoningContextId, owningNodeId: proofTree.id }
            ],
            proofTreeChanges: [{
                newProofTree: {
                    ...proofTree,
                    premisses: [
                        createEmptyVisualProofEditorProofTreeFromProp(existsProp),
                        createEmptyVisualProofEditorProofTreeFromProp(conclusion.value),
                    ],
                    rule: { kind: 'ExistsElim', value: [instantiatedObjectIdent.name, propIdent.name] },
                },
                reasoningContextId,
                nodeId: proofTree.id,
            }],
        };

    }

    protected async handleRuleDownards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const {
            selectedProofTreeNodes,
            generateIdentifier,
            generateUniqueNumber,
        } = params;

        if (selectedProofTreeNodes.length !== 1) {
            throw new Error('Cannot apply this rule on multiple nodes.');
        }

        const { proofTree, reasoningContextId } = selectedProofTreeNodes[0];
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            throw new Error('Conclusion is not an existential quantification');
        }

        const propConclusion = conclusion.value;

        if (propConclusion.kind !== 'Exists') {
            throw new Error('Conclusion is not an existential quantification');
        }

        // ask for new conclusion
        let newConclusion = (await Swal.fire({
            title: 'Enter new conclusion',
            input: 'text',
            inputLabel: 'Conclusion',
            inputPlaceholder: 'C',
            showCloseButton: true,
        })).value;

        if (!newConclusion) {
            return this.createEmptyProofRuleHandlerResult();
        }

        newConclusion = this.parseProp(newConclusion);

        const { object_ident, object_type_ident, body } = propConclusion.value;

        const instantiatedObjectIdent = {
            name: generateIdentifier(),
            unique_id: generateUniqueNumber(),
        };

        const propIdent = {
            name: generateIdentifier(),
            unique_id: generateUniqueNumber(),
        };

        const instantiated_body = instantiate_free_parameter(body, object_ident, instantiatedObjectIdent);

        const nodeId = v4();

        return {
            newReasoningContexts: [],
            removedReasoingContextIds: [],
            additionalAssumptions: [
                { assumption: { kind: 'Datatype', ident: instantiatedObjectIdent, datatype: object_type_ident }, owningReasoningCtxId: reasoningContextId, owningNodeId: nodeId },
                { assumption: { kind: 'PropIsTrue', ident: propIdent, prop: instantiated_body }, owningReasoningCtxId: reasoningContextId, owningNodeId: nodeId }
            ],
            proofTreeChanges: [{
                newProofTree: {
                    id: nodeId,
                    premisses: [
                        { ...proofTree },
                        createEmptyVisualProofEditorProofTreeFromProp(newConclusion),
                    ],
                    rule: { kind: 'ExistsElim', value: [instantiatedObjectIdent.name, propIdent.name] },
                    conclusion: { kind: 'PropIsTrue', value: newConclusion },
                },
                nodeId: proofTree.id,
                reasoningContextId,
            }],
        };
    }
}