import { v4 } from 'uuid';
import Swal from 'sweetalert2';
import { bind_identifier, get_free_parameters, Identifier, instantiate_free_parameter, Prop } from 'alice';
import { VisualProofEditorRuleHandlerParams, ProofRuleHandlerResult } from '..';
import { ProofRuleHandler } from './proof-rule-handler';
import { createEmptyVisualProofEditorProofTreeFromProp, createEmptyVisualProofEditorProofTreeFromTypeJudgment } from '../../lib/visual-proof-editor-proof-tree';
import { VisualProofEditorParameterBindingSelector } from '../../components/visual-proof-editor-parameter-binding-selector';
import React from 'react';
import withReactContent from 'sweetalert2-react-content';
import { createIdentifierGenerator } from './create-identifier-generator';
import { isEqual } from 'lodash';

const ReactSwal = withReactContent(Swal);

export class ForAllElimRuleHandler extends ProofRuleHandler {

    public getLatexCode(): string {
        return `
            \\begin{prooftree}
                \\AxiomC{$\\forall x:\\tau. A(x)$}
                \\AxiomC{$a : \\tau$}
                \\RightLabel{$\\forall E$}
                \\BinaryInfC{$A(a)$}
            \\end{prooftree}
        `;
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
            error('Cannot apply rule on this node.');
            return;
        }

        const prop = conclusion.value;

        let identifier: Identifier | null = null;
        let parameterIndices: Uint32Array = new Uint32Array();
        const prompt = await ReactSwal.fire({
            title: 'Click on the parameters you want to bind.',
            showCloseButton: true,
            html: (
                <VisualProofEditorParameterBindingSelector
                    prop={prop}
                    onSelect={(ident, indices) => {
                        identifier = ident;
                        parameterIndices = new Uint32Array(indices);
                    }}
                />
            )
        });

        if (prompt.isDismissed || prompt.isDenied) {
            return;
        }

        if (!identifier) {
            identifier = await this.promptAssumptionIdent({
                title: 'Select the assumption you apply the universal quantification on.',
                assumptions,
                error,
            });
        }

        if (!identifier) {
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

        // find type name
        const paramAssumption = assumptions
            .find((assumption) => isEqual(assumption.assumption.ident, identifier));

        if (!paramAssumption) {
            throw new Error('Assumption unknown: ' + JSON.stringify(identifier));
        }

        if (paramAssumption.assumption.kind !== 'Datatype') {
            throw new Error('Expected assumption to be datatype.');
        }

        const typeName = paramAssumption.assumption.datatype;

        const boundProp = bind_identifier(
            conclusion.value,
            { kind: 'ForAll' },
            identifier,
            parameterIndices,
            bindIdentifier,
            typeName,
        );

        return {
            additionalAssumptions: [],
            newReasoningContexts: [],
            removedReasoingContextIds: [],
            proofTreeChanges: [{
                newProofTree: {
                    ...proofTree,
                    premisses: [
                        createEmptyVisualProofEditorProofTreeFromProp(boundProp),
                        createEmptyVisualProofEditorProofTreeFromTypeJudgment(identifier, typeName),
                    ],
                    rule: { kind: 'ForAllElim' },
                },
                nodeId: proofTree.id,
                reasoningContextId,
            }],
        };
    }

    protected async handleRuleDownards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined> {
        const { selectedProofTreeNodes, error } = params;

        if (selectedProofTreeNodes.length !== 2) {
            error('Can apply this rule only when both premisses are given.');
            return;
        }

        const [fst, snd] = selectedProofTreeNodes;

        const fstIsProp = fst.proofTree.conclusion.kind === 'PropIsTrue';
        const sndIsProp = snd.proofTree.conclusion.kind === 'PropIsTrue';

        if (fstIsProp && sndIsProp) {
            error('One of the premisses has to be a datatype.');
            return;
        }

        if (!fstIsProp && !sndIsProp) {
            error('One of the premisses has to be an universal quantification');
            return;
        }

        const propPremisse = fstIsProp ? fst : snd;
        const datatypePremisse = fstIsProp ? snd : fst;

        const prop = propPremisse.proofTree.conclusion.value as Prop;
        const datatype = datatypePremisse.proofTree.conclusion.value as [Identifier, string];

        if (prop.kind !== 'ForAll') {
            error('One of the premisses has to be an universal quantification');
            return;
        }

        const { object_ident, body } = prop.value;
        const instantiated_body = instantiate_free_parameter(body, object_ident, datatype[0]);

        return {
            additionalAssumptions: [],
            removedReasoingContextIds: [fst.reasoningContextId, snd.reasoningContextId],
            proofTreeChanges: [],
            newReasoningContexts: [{
                id: v4(),
                selectedNodeId: null,
                isDragging: false,
                x: 0,
                y: 0,
                proofTree: {
                    id: v4(),
                    premisses: [
                        propPremisse.proofTree,
                        datatypePremisse.proofTree,
                    ],
                    rule: { kind: 'ForAllElim' },
                    conclusion: { kind: 'PropIsTrue', value: instantiated_body },
                },
            }],
        };
    }
}