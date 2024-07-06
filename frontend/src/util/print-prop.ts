import { Prop } from 'alice';

export function printProp(prop: Prop): string {
    switch (prop.kind) {
        case 'Atom': return prop.value[0] + (prop.value[1].length > 0 ? `(${prop.value[1].join(', ')})` : '');
        case 'And': return printProp(prop.value[0]) + ' ∧ ' + printProp(prop.value[1]);
        case 'Or': return printProp(prop.value[0]) + ' ∨ ' + printProp(prop.value[1]);
        case 'Impl': return printProp(prop.value[0]) + ' ⊃ ' + printProp(prop.value[1]);
        case 'ForAll': return `∀${prop.value.object_ident}:${prop.value.object_type_ident}. ` + printProp(prop.value.body);
        case 'Exists': return `∃${prop.value.object_ident}:${prop.value.object_type_ident}. ` + printProp(prop.value.body);
        case 'True': return '⊤';
        case 'False': return '⊥';
    }
}