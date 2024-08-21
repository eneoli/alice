import { get_free_parameters, Identifier, instantiate_free_parameter, parse_prop, Prop } from 'alice';
import { AssumptionContext, ProofRuleHandlerResult, SelectedProofTreeNode, VisualProofEditorRuleHandlerParams } from '..';
import Swal from 'sweetalert2';
import { uniq } from 'lodash';
import withReactContent from 'sweetalert2-react-content';
import React from 'react';
import { VisualProofEditorParameterIdentifierSelector } from '../../components/visual-proof-editor-parameter-identifier-selector';

const ReactSwal = withReactContent(Swal);

export type RuleDirection = 'Upwards' | 'Downwards';

interface PromptAssumptionIdentOptions {
    title: string;
    assumptions: AssumptionContext[];
    error: (msg: string) => void;
}

interface PromptPropOptions {
    title: string;
    inputPlaceholder?: string;
    assumptions: AssumptionContext[];
    error: (msg: string) => void;
}

export abstract class ProofRuleHandler {

    public abstract getLatexCode(): string;

    protected abstract handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined>;

    protected abstract handleRuleDownards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined>;

    protected async promptAssumptionIdent(options: PromptAssumptionIdentOptions): Promise<Identifier | null> {
        const { title, assumptions, error } = options;

        const selectOptions = assumptions
            .reduce((accu, current, i) => {
                if (current.assumption.kind !== 'Datatype') {
                    return accu;
                }

                accu.set(i, `${current.assumption.ident.name} : ${current.assumption.datatype}`);

                return accu;
            }, new Map<number, string>());

        if (selectOptions.size == 0) {
            error('There are no witnesses you can select.');
            return null;
        }

        let assumption = (await Swal.fire({
            title,
            input: 'select',
            inputOptions: Object.fromEntries(selectOptions.entries()),
        })).value;

        if (!assumption) {
            return null;
        }
        assumption = assumptions[assumption];

        return assumption.assumption.ident;
    }

    protected async promptProp(options: PromptPropOptions): Promise<Prop | null> {
        const {
            title,
            inputPlaceholder,
            assumptions,
            error,
        } = options;

        let prop = (await Swal.fire({
            title,
            inputPlaceholder,
            input: 'text',
            showCloseButton: true,
        })).value;

        if (!prop) {
            return null;
        }

        prop = this.parseProp(prop);

        let params = get_free_parameters(prop);
        params = uniq(params);

        const needSelectionNames = [];
        for (const param of params) {
            if (param.kind !== 'Uninstantiated') {
                continue;
            }

            const paramName = param.value;

            const paramAssumptions = assumptions
                .filter((assumption) => assumption.assumption.ident.name === paramName);

            if (paramAssumptions.length === 0) {
                error(`Unknown identifier: ${paramName}`);
                return null;
            }

            const datatypeParamAssumptions = paramAssumptions
                .filter((assumption) => assumption.assumption.kind === 'Datatype');

            if (datatypeParamAssumptions.length === 0) {
                error(`Not a value: ${paramName}`);
                return null;
            }

            if (datatypeParamAssumptions.length > 1) {
                needSelectionNames.push(paramName);
                continue;
            }

            prop = instantiate_free_parameter(prop, paramName, datatypeParamAssumptions[0].assumption.ident);
        }

        if (needSelectionNames.length === 0) {
            return prop;
        }

        // prompt for user selection
        const selectionOptions = needSelectionNames
            .reduce((accu, name) => {
                accu[name] = assumptions
                    .filter((assumptions) => assumptions.assumption.ident.name === name)
                    .map((assumption) => assumption.assumption.ident);

                return accu;
            }, {} as { [name: string]: Identifier[] });

        let finalProp: Prop | null = null;
        await ReactSwal.fire({
            title: 'Please further specify which identifier you mean.',
            html: React.createElement(VisualProofEditorParameterIdentifierSelector, {
                prop,
                options: selectionOptions,
                onAllSelect: (prop) => { finalProp = prop; },
                getPopupContainer: () => Swal.getContainer() || document.body,
            }),
        });

        return finalProp;
    }

    protected parseProp(prop: string): Prop {
        try {
            return parse_prop(prop);
        } catch (_) {
            throw new Error('Failed to parse prop.');
        }
    }

    protected createEmptyProofRuleHandlerResult(): ProofRuleHandlerResult {
        return {
            additionalAssumptions: [],
            newReasoningContexts: [],
            proofTreeChanges: [],
            removedReasoingContextIds: []
        };
    }

    protected allNodesAreLeafs(nodes: SelectedProofTreeNode[]): boolean {
        return nodes.every((node) => node.isLeaf);
    }

    protected allNodesAreRoots(nodes: SelectedProofTreeNode[]): boolean {
        return nodes.every((node) => node.isRoot);
    }

    protected allNodesHaveRules(nodes: SelectedProofTreeNode[]): boolean {
        return nodes.every((node) => node.proofTree.rule !== null);
    }

    protected isSingleNodeSelected(nodes: SelectedProofTreeNode[]): boolean {
        return nodes.length === 1;
    }

    public canReasonUpwards(nodes: SelectedProofTreeNode[]): boolean {
        return (
            this.isSingleNodeSelected(nodes) &&
            this.allNodesAreLeafs(nodes) // single selected node is leaf
        );
    };

    public canReasonDownwards(nodes: SelectedProofTreeNode[]): boolean {
        console.log(nodes);
        return (
            this.allNodesAreRoots(nodes)
            // && this.allNodesHaveRules(params) // from willReasonDownwards, can be removed
        );
    }

    public async handleRule(params: VisualProofEditorRuleHandlerParams, direction: RuleDirection): Promise<ProofRuleHandlerResult | undefined> {
        switch (direction) {
            case 'Upwards': return this.handleRuleUpwards(params);
            case 'Downwards': return this.handleRuleDownards(params);
        }
    }
}