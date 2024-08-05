import { Identifier, ProofTreeConclusion, Prop } from 'alice';
import { ProofRuleHandler } from './proof-rule-handler/proof-rule-handler';
import { AndElimFstRuleHandler } from './proof-rule-handler/and-elim-fst-rule-handler';
import { AndElimSndRuleHandler } from './proof-rule-handler/and-elim-snd-rule-handler';
import { OrIntroFstRuleHandler } from './proof-rule-handler/or-intro-fst-rule-handler';
import { ImplIntroRuleHandler } from './proof-rule-handler/impl-intro-rule-handler';
import { TrueIntroRuleHandler } from './proof-rule-handler/true-intro-rule-handler';
import { FalseElimRuleHandler } from './proof-rule-handler/false-elim-rule-handler';
import { ForallIntroRuleHandler } from './proof-rule-handler/forall-intro-rule-handler';
import { OrElimRuleHandler } from './proof-rule-handler/or-elim-rule-handler';
import { ImplElimRuleHandler } from './proof-rule-handler/impl-elim-rule-handler';
import { ExistsElimRuleHandler } from './proof-rule-handler/exists-elim-rule-handler';
import { OrIntroSndRuleHandler } from './proof-rule-handler/or-intro-snd-rule-handler';
import { AndIntroRuleHandler } from './proof-rule-handler/and-intro-rule-handler';
import { VisualProofEditorProofTree } from '../lib/visual-proof-editor-proof-tree';
import { VisualProofEditorReasoningContext } from '../lib/visual-proof-editor-reasoning-context';
import { ExistsIntroRuleHandler } from './proof-rule-handler/exists-intro-rule-handler';
import { ForAllElimRuleHandler } from './proof-rule-handler/forall-elim-rule-handler';

interface SelectedProofTreeNode {
    reasoningContextId: string,
    proofTree: VisualProofEditorProofTree,
    isRoot: boolean,
    isLeaf: boolean,
}

export interface VisualProofEditorRuleHandlerParams {
    selectedProofTreeNodes: SelectedProofTreeNode[],
    assumptions: AssumptionContext[],
    generateIdentifier: () => string,
    generateUniqueNumber: () => number,
}

export interface ProofRuleHandlerProofTreeChange {
    reasoningContextId: string;
    nodeId: string,
    newProofTree: VisualProofEditorProofTree,
}

export type Assumption = { kind: 'PropIsTrue', prop: Prop, ident: Identifier }
    | { kind: 'Datatype', datatype: string, ident: Identifier };

export interface AssumptionContext {
    assumption: Assumption;
    owningReasoningCtxId: string;
    owningNodeId: string;
}

export interface ProofRuleHandlerResult {
    proofTreeChanges: ProofRuleHandlerProofTreeChange[],
    removedReasoingContextIds: string[],
    newReasoningContexts: VisualProofEditorReasoningContext[],
    additionalAssumptions: AssumptionContext[],
}

export type NaturalDeductionRule = 'TrueIntro'
    | 'AndIntro'
    | 'AndElimFst'
    | 'AndElimSnd'
    | 'ImplIntro'
    | 'ImplElim'
    | 'OrIntroFst'
    | 'OrIntroSnd'
    | 'OrElim'
    | 'FalsumElim'
    | 'ForAllIntro'
    | 'ForAllElim'
    | 'ExistsIntro'
    | 'ExistsElim'
    | 'Hypothesis';

export interface VisualProofEditorRule {
    id: NaturalDeductionRule;
    name: string;
    handler: ProofRuleHandler;
}

export function createProofTreeConclusionFromAssumption(assumption: Assumption): ProofTreeConclusion {
    let conclusion: ProofTreeConclusion;
    switch (assumption.kind) {
        case 'PropIsTrue':
            conclusion = { kind: 'PropIsTrue', value: assumption.prop };
            break;
        case 'Datatype':
            conclusion = { kind: 'TypeJudgement', value: [assumption.ident, assumption.datatype] };
            break;
        default: throw new Error('Cannot handle this assumption kind.');
    }

    return conclusion;
}

export function getProofRule(id: string): VisualProofEditorRule {
    for (const rule of NaturalDeductionRules) {
        if (rule.id === id) {
            return rule;
        }
    }

    throw new Error(`Unknown rule ${id}`);
}

export const NaturalDeductionRules: VisualProofEditorRule[] = [
    {
        id: 'TrueIntro',
        name: 'Truth Introduction',
        handler: new TrueIntroRuleHandler(),
    },
    {
        id: 'FalsumElim',
        name: 'Falsum Elimination',
        handler: new FalseElimRuleHandler(),
    },
    {
        id: 'AndIntro',
        name: 'And Introduction',
        handler: new AndIntroRuleHandler(),
    },
    {
        id: 'AndElimFst',
        name: 'And Elimination',
        handler: new AndElimFstRuleHandler(),
    },
    {
        id: 'AndElimSnd',
        name: 'And Elimination',
        handler: new AndElimSndRuleHandler(),
    },
    {
        id: 'ImplIntro',
        name: 'Implication Introduction',
        handler: new ImplIntroRuleHandler(),
    },
    {
        id: 'ImplElim',
        name: 'Implication Elimination',
        handler: new ImplElimRuleHandler(),
    },
    {
        id: 'OrIntroFst',
        name: 'Or Introduction',
        handler: new OrIntroFstRuleHandler(),
    },
    {
        id: 'OrIntroSnd',
        name: 'Or Introduction',
        handler: new OrIntroSndRuleHandler(),
    },
    {
        id: 'OrElim',
        name: 'Or Elimination',
        handler: new OrElimRuleHandler(),
    },
    {
        id: 'ForAllIntro',
        name: 'Universal Quantification Introduction',
        handler: new ForallIntroRuleHandler(),
    },
    {
        id: 'ForAllElim',
        name: 'Universal Quantification Elimination',
        handler: new ForAllElimRuleHandler(),
    },
    {
        id: 'ExistsIntro',
        name: 'Existential Quantification Introduction',
        handler: new ExistsIntroRuleHandler(),
    },
    {
        id: 'ExistsElim',
        name: 'Existential Quantification Elimination',
        handler: new ExistsElimRuleHandler(),
    },
];