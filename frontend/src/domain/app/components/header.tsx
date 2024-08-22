import { css } from '@emotion/css';
import { Button } from 'antd';
import React, { useRef, useState } from 'react';
import { Button as AriaButton, Input, Label, SearchField } from 'react-aria-components';

interface HeaderProps {
    onPropChange: (prop: string) => void;
    onVerify: (prop: string) => void;
    onExportAsOcaml: () => void;
    enableTutor: boolean;
    onTutorClick: () => void;
}

export function Header({ onPropChange, onVerify, onExportAsOcaml, enableTutor, onTutorClick }: HeaderProps) {

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
        replaceSymbol('~', '¬');

        replaceSymbol('\\and', '∧');
        replaceSymbol('&', '∧');

        replaceSymbol('\\or', '∨');
        replaceSymbol('|', '∨');

        replaceSymbol('\\implies', '⊃');
        replaceSymbol('->', '⊃');
        replaceSymbol('=>', '⊃');

        replaceSymbol('\\top', '⊤');
        replaceSymbol('\\bot', '⊥');

        setProp(value);
        onPropChange(value);

        setImmediate(() => inputRef.current?.setSelectionRange(currentPos, currentPos));
    }

    return (
        <div className={cssHeader}>
            <div style={{ display: 'flex' }}>
                <div className={cssHeaderTitleContainer}>
                    <span className={cssHeaderTitle}>🔍 Alice</span>
                    <br />
                    <span className={cssHeaderSubtitle}>A constructive logic proof checker</span>
                </div>
                <Button
                    onClick={onTutorClick}
                    type={'default'}
                    disabled={!enableTutor}
                >
                    💡 Tutor
                </Button>
            </div>
            <div className={cssHeaderContainer}>
                <SearchField style={{ width: 1000 }}>
                    <Label>Proposition</Label>
                    <Input ref={inputRef} spellCheck={false} width={1000} value={prop} onChange={(v) => onInputChange(v.currentTarget.value)} />

                    <AriaButton onPressEnd={() => onInputChange('')}>✕</AriaButton>
                </SearchField>

                <div className={cssButtonContainer}>
                    <Button type={'primary'} onClick={() => onVerify(prop)}>Verify</Button>
                    <Button
                        type={'primary'}
                        onClick={onExportAsOcaml}
                        className={cssOcamlButton}
                    >
                        🐫 Export as OCaml
                    </Button>
                </div>
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

const cssHeaderTitleContainer = css`
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-items: center;
`;

const cssHeaderTitle = css`
    font-size: 35px;
    color: white;
    text-align: center;
    margin-bottom: 10px;
`;

const cssHeaderSubtitle = css`
    text-align: center;
    margin-left: 50px;
    margin-top: -15px;
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

const cssButtonContainer = css`
    margin-top: 17.5px;
    margin-left: 10px;
    display: flex;
    flex-direction: row;
    gap: 10px;
`;

const cssOcamlButton = css`
    background-color: #d45304;
    :hover {
        background-color: #db7537 !important;
    }
    :active {
        background-color: #a64002 !important;
    }
`;