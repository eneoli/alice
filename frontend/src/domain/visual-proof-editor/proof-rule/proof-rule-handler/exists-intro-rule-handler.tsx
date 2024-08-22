import { bind_identifier, get_free_parameters, Identifier, instantiate_free_parameter, Prop } from 'alice';
import Swal from 'sweetalert2';
import { ProofRuleHandlerResult, SelectedProofTreeNode, VisualProofEditorRuleHandlerParams } from '..';
import { ProofRuleHandler } from './proof-rule-handler';
import { v4 } from 'uuid';
import { createEmptyVisualProofEditorProofTreeFromProp, createEmptyVisualProofEditorProofTreeFromTypeJudgment } from '../../lib/visual-proof-editor-proof-tree';
import withReactContent from 'sweetalert2-react-content';
import React from 'react';
import { VisualProofEditorParameterBindingSelector } from '../../components/visual-proof-editor-parameter-binding-selector';
import { createIdentifierGenerator } from './create-identifier-generator';

const ReactSwal = withReactContent(Swal);

export class ExistsIntroRuleHandler extends ProofRuleHandler {
    public getLatexCode(): string {
        return `
            \\begin{prooftree}
                \\AxiomC{$a: \\tau$}
                \\AxiomC{$A(a)$}
                \\RightLabel{$\\exists I$}
                \\BinaryInfC{$\\exists x: \\tau. A(x)$}
            \\end{prooftree}
        `;
    }

    public canReasonUpwards(nodes: SelectedProofTreeNode[]): boolean {
        return (
            super.canReasonUpwards(nodes) &&
            nodes.length === 1 &&
            nodes[0].proofTree.conclusion.kind === 'PropIsTrue' &&
            nodes[0].proofTree.conclusion.value.kind === 'Exists'
        );
    }

    public canReasonDownwards(nodes: SelectedProofTreeNode[]): boolean {
        if (!super.canReasonDownwards(nodes)) {
            return false;
        }

        if (nodes.length !== 2) {
            return false;
        }

        const [fst, snd] = nodes;

        const fstIsTypeJudgment = fst.proofTree.conclusion.kind === 'TypeJudgement';
        const sndIsTypeJudgment = snd.proofTree.conclusion.kind === 'TypeJudgement';

        return fstIsTypeJudgment !== sndIsTypeJudgment;
    }

    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined> {
        const { selectedProofTreeNodes, assumptions, error } = params;

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

        // Ask for assumption

        const identifier = await this.promptAssumptionIdent({
            title: 'Select the witness of the existential quantification.',
            assumptions,
            error,
        });

        if (!identifier) {
            return;
        }

        const { body, object_ident, object_type_ident } = propConclusion.value;
        const instantiated_body = instantiate_free_parameter(body, object_ident, identifier);

        return {
            additionalAssumptions: [],
            removedReasoingContextIds: [],
            newReasoningContexts: [],
            proofTreeChanges: [{
                newProofTree: {
                    id: proofTree.id,
                    premisses: [
                        createEmptyVisualProofEditorProofTreeFromTypeJudgment(
                            identifier,
                            object_type_ident,
                        ),
                        createEmptyVisualProofEditorProofTreeFromProp(instantiated_body),
                    ],
                    rule: { kind: 'ExistsIntro' },
                    conclusion,
                },
                nodeId: proofTree.id,
                reasoningContextId,
            }]
        };
    }

    protected async handleRuleDownards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined> {
        const { selectedProofTreeNodes, error } = params;

        if (selectedProofTreeNodes.length !== 2) {
            error('Need exactly two nodes to form an existential proposition.');
            return;
        }

        const [fst, snd] = selectedProofTreeNodes;

        const fstIsTypeJudgment = fst.proofTree.conclusion.kind === 'TypeJudgement';
        const sndIsTypeJudgment = snd.proofTree.conclusion.kind === 'TypeJudgement';

        if (fstIsTypeJudgment && sndIsTypeJudgment) {
            error('Cannot combine two type judgments to form an existential propositon.');
            return;
        }

        if (!fstIsTypeJudgment && !sndIsTypeJudgment) {
            error('Cannot combine two propositions to form an existential proposition.');
            return;
        }

        const typeJudgmentNode = fstIsTypeJudgment ? fst : snd;
        const propJudgmentNode = fstIsTypeJudgment ? snd : fst;

        const prop = propJudgmentNode.proofTree.conclusion.value as Prop;
        const identifier = (typeJudgmentNode.proofTree.conclusion.value as [Identifier, string])[0];
        const typeName = (typeJudgmentNode.proofTree.conclusion.value as [Identifier, string])[1];

        // ask to bind identifiers

        let parameterIndices: Uint32Array = new Uint32Array();
        const prompt = await ReactSwal.fire({
            title: 'Click on the parameters you want to bind.',
            showCloseButton: true,
            html: (
                <VisualProofEditorParameterBindingSelector
                    prop={prop}
                    identifier={identifier}
                    onSelect={(_, indices) => { parameterIndices = new Uint32Array(indices); }}
                />
            )
        });

        if (prompt.isDismissed || prompt.isDenied) {
            return;
        }

        const freeParams = get_free_parameters(prop);
        const freeParamNames = freeParams
            .map((ident) => ident.kind === 'Instantiated' ? ident.value.name : ident.value);

        const generateLocalIdentifier = createIdentifierGenerator('xyzuvwabcdefghijklmnopqrst'.split(''));

        let bindIdentifier = generateLocalIdentifier();
        while (freeParamNames.includes(bindIdentifier)) {
            bindIdentifier = generateLocalIdentifier();
        }

        const boundProp = bind_identifier(
            prop,
            { kind: 'Exists' },
            identifier,
            parameterIndices,
            bindIdentifier,
            typeName,
        );

        return {
            additionalAssumptions: [],
            proofTreeChanges: [],
            removedReasoingContextIds: [fst.reasoningContextId, snd.reasoningContextId],
            newReasoningContexts: [{
                id: v4(),
                selectedNodeId: null,
                isDragging: false,
                x: 0,
                y: 0,
                proofTree: {
                    id: v4(),
                    premisses: [typeJudgmentNode.proofTree, propJudgmentNode.proofTree],
                    rule: { kind: 'ExistsIntro' },
                    conclusion: { kind: 'PropIsTrue', value: boundProp },
                }
            }],
        };
    }
}
