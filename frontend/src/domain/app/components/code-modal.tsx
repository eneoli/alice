import React from 'react';
import { Modal } from 'antd';
import { CopyBlock } from 'react-code-blocks';

interface CodeModalProps {
    title: string;
    code: string;
    language: string;
    onClose: () => void;
}

export function CodeModal(props: CodeModalProps) {

    const { title, code, language, onClose } = props;

    return (
        <Modal
            style={{ fontFamily: 'monospace' }}
            width={'fit-content'}
            title={title}
            open={true}
            onCancel={onClose}
            onClose={onClose}
            footer={[]}>
            <CopyBlock
                text={code}
                language={language}
                showLineNumbers={true}
                codeBlock={true}
                codeContainerStyle={{ marginTop: '25px' }}
                theme={theme} />
        </Modal>
    );
}

const theme = {
    lineNumberColor: '#666666',
    lineNumberBgColor: '#f2f2f2',
    backgroundColor: '#f7f7f7',
    textColor: '#222222',
    substringColor: '#c41a16',
    keywordColor: '#005f9e',
    attributeColor: '#d9931e',
    selectorAttributeColor: '#7a1e80',
    docTagColor: '#555555',
    nameColor: '#267f99',
    builtInColor: '#e06c00',
    literalColor: '#b00000',
    bulletColor: '#e06c00',
    codeColor: '#0000ff',
    additionColor: '#007d3c',
    regexpColor: '#c41a16',
    symbolColor: '#a67f59',
    variableColor: '#a67f59',
    templateVariableColor: '#a67f59',
    linkColor: '#005f9e',
    selectorClassColor: '#267f99',
    typeColor: '#008080',
    stringColor: '#c41a16',
    selectorIdColor: '#7a1e80',
    quoteColor: '#007d3c',
    templateTagColor: '#e06c00',
    deletionColor: '#a00000',
    titleColor: '#1e90ff',
    sectionColor: '#1e90ff',
    commentColor: '#556b2f',
    metaKeywordColor: '#1e90ff',
    metaColor: '#1e90ff',
    functionColor: '#795e26',
    numberColor: '#007d3c',
};