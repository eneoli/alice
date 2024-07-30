import { Identifier, Prop, PropParameter } from 'alice';

export function printTypeJudgment([ident, datatype]: [Identifier, string]): string {
    return `${ident.name} : ${datatype}`;
}

export function printProp(prop: Prop): string {

    const wrap = (prop: Prop, shouldWrap: boolean) => {
        if (shouldWrap) {
            return `(${printProp(prop)})`;
        }

        return printProp(prop);
    };

    if (prop.kind === 'Atom') {
        return prop.value[0] + (prop.value[1].length > 0 ? `(${prop.value[1].map(printParam).join(', ')})` : '');
    }

    if (prop.kind === 'True') {
        return '⊤';
    }

    if (prop.kind === 'False') {
        return '⊥';
    }

    if (prop.kind === 'ForAll') {
        return `∀${prop.value.object_ident}:${prop.value.object_type_ident}. ` + printProp(prop.value.body);
    }

    if (prop.kind === 'Exists') {
        return `∃${prop.value.object_ident}:${prop.value.object_type_ident}. ` + printProp(prop.value.body);
    }

    let connective;
    switch (prop.kind) {
        case 'And':
            connective = '∧';
            break;
        case 'Or':
            connective = '∨';
            break;
        case 'Impl':
            connective = '⊃';
    }

    const [fst, snd] = prop.value;

    const propPrecedence = getPrecedence(prop);
    const fstPrecedence = getPrecedence(fst);
    const sndPrecedence = getPrecedence(snd);

    const shouldWrapFst = (propPrecedence > fstPrecedence) || (propPrecedence === fstPrecedence && isRightAssociative(prop));
    const shouldWrapSnd = (propPrecedence > sndPrecedence) || (propPrecedence === sndPrecedence && isLeftAssociative(prop));

    return wrap(fst, shouldWrapFst) + ` ${connective} ` + wrap(snd, shouldWrapSnd);
}

function getPrecedence(prop: Prop): number {
    switch (prop.kind) {
        case 'Atom': return 999;
        case 'True': return 999;
        case 'False': return 999;
        case 'And': return 4;
        case 'Or': return 3;
        case 'Impl': return 2;
        case 'ForAll': return 1;
        case 'Exists': return 1;
    }
}

function isLeftAssociative(prop: Prop): boolean {
    switch (prop.kind) {
        case 'Atom': return false;
        case 'True': return false;
        case 'False': return false;
        case 'And': return true;
        case 'Or': return true;
        case 'Impl': return false;
        case 'ForAll': return false;
        case 'Exists': return false;
    }
}

function isRightAssociative(prop: Prop): boolean {
    switch (prop.kind) {
        case 'Atom': return false;
        case 'True': return false;
        case 'False': return false;
        case 'And': return false;
        case 'Or': return false;
        case 'Impl': return true;
        case 'ForAll': return false;
        case 'Exists': return false;
    }
}

function printParam(param: PropParameter): string {
    switch (param.kind) {
        case 'Uninstantiated': return param.value;
        case 'Instantiated': return param.value.name;
    }
}