import { Prop } from 'alice';
import { VisualProofEditorProofTree } from '../components/visual-proof-editor';
import { handleAndElimFstRule } from './proof-rule-handler/handle-and-elim-fst-rule';
import { handleAndElimSndRule } from './proof-rule-handler/handle-and-elim-snd-rule';
import { handleAndIntroRule } from './proof-rule-handler/handle-and-intro-rule';
import { handleExistsElimRule } from './proof-rule-handler/handle-exists-elim-rule';
import { handleExistsIntroRule } from './proof-rule-handler/handle-exists-intro-rule';
import { handleFalsumElimRule } from './proof-rule-handler/handle-falsum-elim-rule';
import { handleForAllElimRule } from './proof-rule-handler/handle-forall-elim-rule';
import { handleForAllIntroRule } from './proof-rule-handler/handle-forall-intro-rule';
import { handleImplElimRule } from './proof-rule-handler/handle-impl-elim-rule';
import { handleImplIntroRule } from './proof-rule-handler/handle-impl-intro-rule';
import { handleOrElimRule } from './proof-rule-handler/handle-or-elim-rule';
import { handleOrIntroFstRule } from './proof-rule-handler/handle-or-intro-fst-rule';
import { handleOrIntroSndRule } from './proof-rule-handler/handle-or-intro-snd-rule';
import { handleTrueIntroRule } from './proof-rule-handler/handle-true-intro-rule';

export interface VisualProofEditorRuleHandlerParams {
    proofTree: VisualProofEditorProofTree,
    reasoningContextId: string,
    generateIdentifier: () => string,
}

export type Assumption = { kind: 'PropIsTrue', prop: Prop, ident: string } | { kind: 'Datatype', datatype: string, ident: string };

export interface AssumptionContext {
    assumption: Assumption;
    owningReasoningCtxId: string;
    owningNodeId: string;
}

export interface ProofRuleHandlerResult {
    newProofTree: VisualProofEditorProofTree,
    additionalAssumptions: AssumptionContext[],
}

type VisualProofEditorRuleHandler = (params: VisualProofEditorRuleHandlerParams) => Promise<ProofRuleHandlerResult>;

export type NaturalDeductionRule = 'TrueIntro' | 'AndIntro' | 'AndElimFst' | 'AndElimSnd' | 'ImplIntro' | 'ImplElim' | 'OrIntroFst' | 'OrIntroSnd' | 'OrElim' | 'FalsumElim' | 'ForAllIntro' | 'ForAllElim' | 'ExistsIntro' | 'ExistsElim' | 'Hypothesis';

export interface VisualProofEditorRule {
    id: NaturalDeductionRule;
    name: string;
    reasoning: 'TopDown' | 'BottomUp';
    handler: VisualProofEditorRuleHandler;
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
        reasoning: 'BottomUp',
        handler: handleTrueIntroRule,
    },
    {
        id: 'FalsumElim',
        name: 'Falsum Elim',
        reasoning: 'BottomUp',
        handler: handleFalsumElimRule,
    },
    {
        id: 'AndIntro',
        name: 'And Intro',
        reasoning: 'BottomUp',
        handler: handleAndIntroRule,
    },
    {
        id: 'AndElimFst',
        name: 'And Elim Fst',
        reasoning: 'TopDown',
        handler: handleAndElimFstRule,
    },
    {
        id: 'AndElimSnd',
        name: 'And Elim Snd',
        reasoning: 'TopDown',
        handler: handleAndElimSndRule,
    },
    {
        id: 'ImplIntro',
        name: 'Implication Introduction',
        reasoning: 'BottomUp',
        handler: handleImplIntroRule,
    },
    {
        id: 'ImplElim',
        name: 'Implication Elimination',
        reasoning: 'TopDown',
        handler: handleImplElimRule,
    },
    {
        id: 'OrIntroFst',
        name: 'Or Introduction Fst',
        reasoning: 'BottomUp',
        handler: handleOrIntroFstRule,
    },
    {
        id: 'OrIntroSnd',
        name: 'Or Introduction Snd',
        reasoning: 'BottomUp',
        handler: handleOrIntroSndRule,
    },
    {
        id: 'OrElim',
        name: 'Or Elimination',
        reasoning: 'TopDown',
        handler: handleOrElimRule,
    },
    {
        id: 'ForAllIntro',
        name: 'Universal quantification Introduction',
        reasoning: 'BottomUp',
        handler: handleForAllIntroRule,
    },
    {
        id: 'ForAllElim',
        name: 'Universal quantification Elimination',
        reasoning: 'TopDown',
        handler: handleForAllElimRule,
    },
    {
        id: 'ExistsIntro',
        name: 'Existential quantification Introduction',
        reasoning: 'BottomUp',
        handler: handleExistsIntroRule,
    },
    {
        id: 'ExistsElim',
        name: 'Existential quantification Elimination',
        reasoning: 'TopDown',
        handler: handleExistsElimRule,
    },
];