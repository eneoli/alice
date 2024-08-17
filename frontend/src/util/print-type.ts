import { print_prop, Type } from 'alice';

export function printType(type: Type) {
    switch (type.kind) {
        case 'Prop': return print_prop(type.value);
        case 'Datatype': return `${type.value[0]}: ${type.value[1]}`;
    }
}