import { css } from '@emotion/css';
import { CheckError, PropKind, SynthesizeError, Type } from 'alice';
import React from 'react';
import { printType } from '../../../util/print-type';

interface TutorTypeCheckErrorDisplayProps {
    code: string;
    error: CheckError;
}

export function TutorTypeCheckErrorDisplay(props: TutorTypeCheckErrorDisplayProps) {

    const { code } = props;
    const { error, kind } = unboxTypeCheckerError(props.error, 'CheckError');

    const span = (span: Span | null) => {
        if (!span) {
            return null;
        }

        const { line, column } = getLineAndColumn(code, span.start);

        return (
            <span style={{ color: '#B3B3B3' }}>
                line: {line}, column: {column}
            </span>
        );
    };

    return (
        <div className={cssTypeCheckErrorDisplayContainer}>
            {
                kind === 'CheckError' && (
                    <span className={cssHeading}>
                        💥 Type Checking failed:
                    </span>
                )
            }
            {
                kind === 'SynthesizeError' && (
                    <span className={cssHeading}>
                        💥 Type Inference failed:
                    </span>
                )
            }

            <ul className={cssErrorList}>
                {
                    error.kind === 'UnknownIdentifier' && (
                        <li>
                            {ei('UnknownIdentifier')}
                            cannot find value `{error.value[0]}` in this scope.

                            <br />
                            {span(error.value[1])}
                        </li>
                    )
                }
                {
                    error.kind === 'TypeAnnotationsNeeded' && (
                        <li>
                            {ei('TypeAnnotationsNeeded')}
                            type annotations needed.

                            <br />
                            {span(error.value)}
                        </li>
                    )
                }
                {
                    error.kind === 'CaseArmsDifferent' && (
                        <li>
                            {ei('CaseArmsDifferent')}
                            The arms of this case expression have different types:
                            <br />
                            <ul>
                                <li>
                                    First arm has type <span style={{ color: '#BA90DF' }}>
                                        {printType(error.value.fst_type)}.
                                    </span>
                                </li>
                                <li>
                                    Second arm has type <span style={{ color: '#BA90DF' }}>
                                        {printType(error.value.snd_type)}
                                    </span>.
                                </li>
                            </ul>
                            <br />
                            {span(error.value.span)}
                        </li>
                    )
                }
                {
                    error.kind === 'NotSynthesizing' && (
                        <li>
                            {ei('NotSynthesizing')}
                            This proof term does not yield an unambiguously type on its own.
                            <br />
                            <br />
                            {hint('Consider adding a type ascription.')}
                            <br />
                            {span(error.value[1])}
                        </li>
                    )
                }
                {
                    error.kind === 'PropHasFreeParameters' && (
                        <li>
                            {ei('PropHasFreeParameters')}
                            The proposition you provided has free parameters.
                            You cannot prove this proposition.
                        </li>
                    )
                }
                {
                    error.kind === 'UnexpectedPropKind' && (
                        <li>
                            {ei('UnexpectedPropKind')}
                            Expected this proof term to have one of the following kinds: {propKinds(error.value.expected)} but received type {showType(error.value.received)}.
                            <br />
                            <br />
                            {span(error.value.span)}
                        </li>
                    )
                }
                {
                    error.kind === 'UnexpectedType' && (
                        <li>
                            {ei('UnexpectedType')}
                            Expected this proof term to have type {showType(error.value.expected)}, but has type {showType(error.value.received)}.
                            <br />
                            <br />
                            {span(error.value.span)}
                        </li>
                    )
                }
                {
                    error.kind === 'UnexpectedTypeAscription' && (
                        <li>
                            {ei('UnexpectedTypeAscription')}
                            The type ascription you provided is contradictory to the expected type.
                            <br />
                            Type ascription: {showType(error.value.ascription)}
                            <br />
                            Expected type: {showType(error.value.expected)}
                            <br />
                            <br />
                            {hint('You can remove this type ascription.')}
                            <br />
                            {span(error.value.span)}
                        </li>
                    )
                }
                {
                    error.kind === 'IncompatibleProofTerm' && (
                        <li>
                            {ei('IncompatibleProofTerm')}
                            Expected this proof term to have type {showType(error.value.expected_type)}, but {error.value.proof_term.kind} cannot have this type.
                            <br />
                            {span(error.value.span)}
                        </li>
                    )
                }
                {
                    error.kind === 'QuantifiedObjectEscapesScope' && (
                        <li>
                            {ei('QuantifiedObjectEscapesScope')}
                            The witness of an existential quantification is exposed in the conclusion.
                            <br />
                            <br />
                            {span(error.value)}
                        </li>
                    )
                }
                {
                    error.kind === 'ExpectedPropAsSecondPairComponent' && (
                        <li>
                            {ei('ExpectedPropAsSecondPairComponent')}
                            The second component of a pair has always to be a proposition.
                            <br />
                            <br />
                            {span(error.value.span)}
                        </li>
                    )
                }
                {
                    error.kind === 'CannotReturnDatatype' && (
                        <li>
                            {ei('CannotReturnDatatype')}
                            This expression is expected to return a datatype. At the current state of
                            the type checker, only identifiers can be treatened as datatype values.
                            <br />
                            <br />
                            {span(error.value)}
                        </li>
                    )
                }
            </ul>
        </div>
    );
}

type Position = { line: number, column: number };

function getLineAndColumn(code: string, position: number): Position {
    const lines = code.substring(0, position).split('\n');
    const line = lines.length;
    const column = lines[lines.length - 1].length + 1;
    return { line, column };
}

const ei = (identifier: string) => (
    <span>
        <span className={cssErrorIdentifier}>{identifier}</span>:&nbsp;
    </span>
);

const showType = (type: Type) => (
    <span style={{ color: '#BA90DF' }}>
        {printType(type)}
    </span>
);

const hint = (hint: string) => (
    <span>
        <span style={{ color: '#50B498' }}>Hint</span>:&nbsp;
        {hint}
    </span>
);

const propKinds = (kinds: PropKind[]) => kinds.map((kind, i) => (
    <>
        {i !== 0 && (<span key={i + '-comma'}>, </span>)}
        <span style={{ color: '#BA90DF' }} key={i}>{kind.kind}</span>
    </>
));

type Span = {
    start: number,
    end: number,
};

const cssTypeCheckErrorDisplayContainer = css`
    display: flex;
    flex-direction: column;
    align-items: flex-start;
`;

const cssHeading = css`
    color: white;
    font-size: 1.5em;
`;

const cssErrorList = css`
    font-size: 1.15em;
    font-weight: bold;
`;

const cssErrorIdentifier = css`
    font-weight: bold;
    color: #EE3B38;
`;

type ErrorKind = 'CheckError' | 'SynthesizeError';

interface UnboxTypeCheckerErrorResult {
    error: CheckError | SynthesizeError;
    kind: ErrorKind;
}

function unboxTypeCheckerError(error: CheckError | SynthesizeError, currentKind: ErrorKind): UnboxTypeCheckerErrorResult {
    // kind refers to the ADT Constructor,
    // not whether error istelf is CheckError or SynthesizeError
    if (error.kind === 'CheckError' || error.kind === 'SynthesizeError') {
        const newKind = currentKind === 'CheckError' ? 'SynthesizeError' : 'CheckError';
        return unboxTypeCheckerError(error.value, newKind);
    }

    return { error, kind: currentKind };
}