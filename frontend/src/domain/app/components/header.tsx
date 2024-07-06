import { css } from '@emotion/css';
import React, { useRef, useState } from 'react';
import { Button, Input, Label, SearchField } from 'react-aria-components';

interface HeaderProps {
    onPropChange: (prop: string) => void;
    onVerify: (prop: string) => void;
}

export function Header({ onPropChange, onVerify }: HeaderProps) {

    const inputRef = useRef<HTMLInputElement>(null);

    const [prop, setProp] = useState('');

    const onInputChange = (value: string) => {
        let currentPos = inputRef.current?.selectionStart || 0;

        const replaceSymbol = (symbol: string, replacement: string) => {

            const matches = (value.substring(0, currentPos + 1).match(new RegExp(symbol.replaceAll('\\', '\\\\'), 'g')) || []).length

            currentPos -= (symbol.length - replacement.length) * matches;

            value = value.replaceAll(symbol, replacement);
        };

        replaceSymbol('\\forall', '∀');
        replaceSymbol('\\forall', '∀');

        replaceSymbol('\\exists', '∃');

        replaceSymbol('\\not', '¬');
        replaceSymbol('!', '¬');

        replaceSymbol('\\and', '∧');
        replaceSymbol('&', '∧');

        replaceSymbol('\\or', '∨');
        replaceSymbol('|', '∨');

        replaceSymbol('\\implies', '→');
        replaceSymbol('->', '→');
        replaceSymbol('=>', '→');

        setProp(value);
        onPropChange(value);

        setImmediate(() => inputRef.current?.setSelectionRange(currentPos, currentPos));
    }

    return (
        <div className={cssHeader}>
            <span className={cssHeaderTitle} style={{ fontSize: 40 }}>🔍 Alice</span>
            <br />
            <span className={cssHeaderSubtitle}>A constructive logic proof checker</span>
            <div className={cssHeaderContainer}>
                <SearchField style={{ width: 1000 }}>
                    <Label>Proposition</Label>
                    <Input ref={inputRef} spellCheck={false} width={1000} value={prop} onChange={(v) => onInputChange(v.currentTarget.value)} />

                    <Button onPressEnd={() => onInputChange('')}>✕</Button>
                </SearchField>

                <Button style={{ marginTop: 18, marginLeft: 10 }}
                    onPressEnd={() => onVerify(prop)}
                >Verify</Button>
            </div>
        </div>
    );
}

const cssHeader = css`
    box-sizing: border-box;
    width: 100%;
    min-height: 75px;
    background-color: #233348;
    padding: 10px;
`;

const cssHeaderTitle = css`
    color: white;
    text-align: center;
`;

const cssHeaderSubtitle = css`
    color: #dfdfdf;
`;

const cssHeaderContainer = css`
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    flex-direction: row;
    color: white;
`;

