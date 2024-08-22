import { ProofTreeRule } from 'alice';

export function printProofRule(proofTreeRule: ProofTreeRule): string {
    switch (proofTreeRule.kind) {
        case 'AndIntro': return '\\land I';
        case 'AndElimFst': return '\\land E_1';
        case 'AndElimSnd': return '\\land E_2';
        case 'TrueIntro': return '\\top I';
        case 'ImplIntro': return '{\\supset}I^' + proofTreeRule.value.name;
        case 'ImplElim': return '\\supset E';
        case 'Ident': return proofTreeRule.value.name;
        case 'OrIntroFst': return '\\lor I_1';
        case 'OrIntroSnd': return '\\lor I_2';
        case 'OrElim': return `\\lor E^{${proofTreeRule.value[0].name}, ${proofTreeRule.value[1].name}}`;
        case 'FalsumElim': return '\\bot E';
        case 'ForAllIntro': return '\\forall I^' + proofTreeRule.value.name;
        case 'ForAllElim': return '\\forall E';
        case 'ExistsIntro': return '\\exists I';
        case 'ExistsElim': return `\\exists E^{${proofTreeRule.value[0].name}, ${proofTreeRule.value[1].name}}`;
        case 'Sorry': return 'sorry';
        case 'AlphaEquivalent': return ' \\alpha\\text{-Eq}'
    }
}