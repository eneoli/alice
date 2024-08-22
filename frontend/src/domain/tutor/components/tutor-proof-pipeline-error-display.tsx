import { css } from '@emotion/css';
import { print_prop, ProofPipelineError } from 'alice';
import React from 'react';

interface TutorProofPipelineErrorDisplayProps {
    error: ProofPipelineError;
}

export function TutorProofPipelineErrorDisplay(props: TutorProofPipelineErrorDisplayProps) {
    const { error } = props;

    if (error.kind === 'UnexpectedProcessingState') {
        return '[UnexpectedProcessingState]: This is likely a bug in Alice.';
    }

    const resolveDatatypesError = error.value.value;

    return (
        <>
            <span className={cssHeading}>
                There was an error while processing your proof.
            </span>
            <ul className={cssErrorList}>
                {
                    resolveDatatypesError.kind === 'AtomUnknown' && (
                        <li>
                            {ei('AtomUnknown')}
                            Atom `{resolveDatatypesError.value}` is unknown.
                            <br />
                            <br />
                            <span style={{ color: '#50B498' }}>Hint</span>: Add a declaration: `atom {resolveDatatypesError.value}( /* arity */ );`
                        </li>
                    )
                }
                {
                    resolveDatatypesError.kind === 'DatatypeUnknown' && (
                        <li>
                            {ei('DatatypeUnknown')}
                            Datatype `{resolveDatatypesError.value}` is unknown.
                            <br />
                            <br />
                            <span style={{ color: '#50B498' }}>Hint</span>: Add a declaration: `datatype {resolveDatatypesError.value};`
                        </li>
                    )
                }
                {
                    resolveDatatypesError.kind === 'DuplicateIdentifier' && (
                        <li>
                            {ei('DuplicateIdentifier')}
                            `{resolveDatatypesError.value}` is defined multiple times.
                        </li>
                    )
                }
                {
                    resolveDatatypesError.kind === 'ArityWrong' && (
                        <li>
                            {ei('ArityWrong')}
                            You used Atom `{resolveDatatypesError.value.ident}` with {resolveDatatypesError.value.actual} parameters, but it only allows {resolveDatatypesError.value.expected} parameters.
                        </li>
                    )
                }
                {
                    resolveDatatypesError.kind === 'PropContainsDatatypeIdentifier' && (
                        <li>
                            {ei('PropContainsDatatypeIdentifier')}
                            The proposition `{print_prop(resolveDatatypesError.value.prop)}` contains datatypes. This is forbidden.
                        </li>
                    )
                }
            </ul>
        </>
    );
}

const ei = (identifier: string) => (
    <span>
        <span className={cssErrorIdentifier}>{identifier}</span>:&nbsp;
    </span>
);

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