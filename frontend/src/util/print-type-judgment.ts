import { Identifier } from 'alice';

export function printTypeJudgment([ident, datatype]: [Identifier, string]): string {
    return `${ident.name} : ${datatype}`;
}