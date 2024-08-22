import { instantiate_free_parameter } from 'alice';
import { v4 } from 'uuid';
import { VisualProofEditorRuleHandlerParams, ProofRuleHandlerResult, SelectedProofTreeNode } from '..';
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

    public canReasonUpwards(nodes: SelectedProofTreeNode[]): boolean {
        return (
            super.canReasonUpwards(nodes) &&
            nodes.length === 1 &&
            nodes[0].proofTree.conclusion.kind === 'PropIsTrue'
        );
    }

    public canReasonDownwards(nodes: SelectedProofTreeNode[]): boolean {
        return (
            super.canReasonDownwards(nodes) &&
            nodes.length === 1 &&
            nodes[0].proofTree.conclusion.kind === 'PropIsTrue' &&
            nodes[0].proofTree.conclusion.value.kind === 'Exists'
        );
    }

    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined> {
        const {
            selectedProofTreeNodes,
            generateIdentifier,
            generateUniqueNumber,
            assumptions,
            error,
        } = params;

        if (selectedProofTreeNodes.length !== 1) {
            error('Cannot apply this rule on multiple nodes.');
            return;
        }

        const { proofTree, reasoningContextId } = selectedProofTreeNodes[0];
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            error('Cannot apply rule on this node.');
            return;
        }

        const existsProp = await this.promptProp({
            title: 'Enter existential quantification you want to eliminate.',
            inputPlaceholder: '∃x:t. A(x)',
            assumptions,
            error,
        });

        if (!existsProp) {
            return;
        }

        if (existsProp.kind !== 'Exists') {
            error('You did not enter an existential quantification.');
            return;
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
                    rule: { kind: 'ExistsElim', value: [instantiatedObjectIdent, propIdent] },
                },
                reasoningContextId,
                nodeId: proofTree.id,
            }],
        };

    }

    protected async handleRuleDownards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined> {
        const {
            selectedProofTreeNodes,
            generateIdentifier,
            generateUniqueNumber,
            assumptions,
            error,
        } = params;

        if (selectedProofTreeNodes.length !== 1) {
            error('Cannot apply this rule on multiple nodes.');
            return;
        }

        const { proofTree, reasoningContextId } = selectedProofTreeNodes[0];
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            error('Conclusion is not an existential quantification');
            return;
        }

        const propConclusion = conclusion.value;

        if (propConclusion.kind !== 'Exists') {
            error('Conclusion is not an existential quantification');
            return;
        }

        const newConclusion = await this.promptProp({
            title: 'Enter new conclusion',
            inputPlaceholder: 'C',
            assumptions,
            error,
        });

        if (!newConclusion) {
            return;
        }

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
                    rule: { kind: 'ExistsElim', value: [instantiatedObjectIdent, propIdent] },
                    conclusion: { kind: 'PropIsTrue', value: newConclusion },
                },
                nodeId: proofTree.id,
                reasoningContextId,
            }],
        };
    }
}