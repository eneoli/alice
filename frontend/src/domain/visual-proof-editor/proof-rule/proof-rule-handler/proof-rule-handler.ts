import { parse_prop, Prop } from 'alice';
import { ProofRuleHandlerResult, VisualProofEditorRuleHandlerParams } from '..';

export abstract class ProofRuleHandler {
    protected abstract handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult>;

    protected abstract handleRuleDownards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult>;

    protected parseProp(prop: string): Prop {
        try {
            return parse_prop(prop);
        } catch (_) {
            throw new Error('Failed to parse prop.');
        }
    }

    protected createEmptyProofRuleHandlerResult(): ProofRuleHandlerResult {
        return { additionalAssumptions: [] };
    }

    public willReasonDownwards(params: VisualProofEditorRuleHandlerParams): boolean {
        const { isRoot, proofTree } = params;
        const hasRule = proofTree.rule !== null;

        return isRoot && hasRule;
    }

    public willReasonUpwards(params: VisualProofEditorRuleHandlerParams): boolean {
        const { isLeaf } = params;

        return !this.willReasonDownwards(params) && isLeaf;
    }

    public async handleRule(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        if (this.willReasonDownwards(params)) {
            return this.handleRuleDownards(params);
        }

        if (this.willReasonUpwards(params)) {
            return this.handleRuleUpwards(params);
        }

        throw new Error('Cannot reason on this node.');
    }
}