import React, { useState } from 'react';
import { Header } from './header';
import { CodeEditor } from '../../code-editor/components/code-editor';
import { VisualProofEditor } from '../../visual-proof-editor/components/visual-proof-editor';
import { ConfigProvider, Drawer, message, theme as antdTheme, ThemeConfig } from 'antd';
import { Prop, export_as_ocaml, parse_prop, verify } from 'alice';
import { debounce, isEqual } from 'lodash';
import { CodeModal } from './code-modal';
import { VisualProofEditorProofTree } from '../../visual-proof-editor/lib/visual-proof-editor-proof-tree';
import { MathJax3Config, MathJaxContext } from 'better-react-mathjax';
import mathjax from 'mathjax/es5/tex-svg';
import bussproofs from 'mathjax/es5/input/tex/extensions/bussproofs'

const mathjaxConfig: MathJax3Config = {
    loader: {
        paths: { app: '/' },
        load: ['output/svg', bussproofs],
    },
    tex: {
        packages: { '[+]': ['bussproofs'] },
    },
    svg: { fontCache: 'global' },
    options: {
        enableMenu: false,
    },
};

export function App() {

    const [proofTerm, setProofTerm] = useState('');
    const [prop, setProp] = useState<Prop | null>(null);
    const [showCodeExport, setShowCodeExport] = useState(false);
    const [showTutor, setShowTutor] = useState(false);
    const [_messageApi, contextHolder] = message.useMessage();

    const handlePropChange = debounce((propString: string) => {
        try {
            const newProp = parse_prop(propString);

            if (!isEqual(prop, newProp)) {
                setProp(newProp);
            }

            setProofTerm('sorry');
        } catch (e) {
            setProp(null);
            console.error(e);
        }
    }, 500);


    const handleProofTreeChange = (proofTree: VisualProofEditorProofTree) => {
        const code = generateCode(proofTree);
        console.log(code);
        setProofTerm(code);
    };

    const handleVerify = (prop: string) => {
        let isProof = false;
        try {
            verify(prop, proofTerm);
            isProof = true;
        } catch (e) {
            console.error(e);
        }

        if (isProof) {
            message.success('Your proof is correct! Well done.');
        } else {
            message.error('Your proof is wrong.');
        }
    };

    const handleOcamlExport = () => {
        if (!prop) {
            return;
        }

        setShowCodeExport(true);
    };

    return (
        <ConfigProvider theme={theme}>
            <MathJaxContext
                src={mathjax}
                config={mathjaxConfig}
                version={3}>
                {contextHolder}
                <Header
                    onPropChange={handlePropChange}
                    onVerify={handleVerify}
                    onExportAsOcaml={handleOcamlExport}
                    onTutorClick={() => setShowTutor(true)}
                />

                {prop && (
                    <>
                        <VisualProofEditor prop={prop} onProofTreeChange={handleProofTreeChange} />

                        <div style={{ marginTop: 20 }}>
                            <CodeEditor height={'20vh'} initialValue={proofTerm} onChange={setProofTerm} />
                        </div>
                    </>
                )}

                {!prop && (
                    <div style={{ textAlign: 'center', color: '#192434' }}>
                        <h1>Alice is ready.</h1>
                        <h2>Please enter a proposition to begin.</h2>
                    </div>
                )}

                {
                    (showCodeExport && prop) && (
                        <CodeModal
                            title='🐫 OCaml Export'
                            code={export_as_ocaml(prop, proofTerm)}
                            language='ocaml'
                            onClose={() => { setShowCodeExport(false) }}
                        />
                    )
                }
                <Drawer title={'💡 Tutor'} open={showTutor} onClose={() => setShowTutor(false)}>
                    Hallo!
                </Drawer>
            </MathJaxContext>
        </ConfigProvider>
    );
}

const theme: ThemeConfig = {
    algorithm: antdTheme.darkAlgorithm,
    token: {
        colorPrimary: '#006af5',
        colorBgBase: '#233348',
        colorPrimaryBg: 'transparent',
    },
};

const generateCode: (proofTree: VisualProofEditorProofTree) => string = (proofTree: VisualProofEditorProofTree) => {

    if (proofTree.rule === null) {
        return 'sorry';
    }

    const rule = proofTree.rule;

    switch (rule.kind) {
        case 'TrueIntro': return '()';
        case 'Ident': return rule.value;
        case 'AndIntro': return `(${generateCode(proofTree.premisses[0])}, ${generateCode(proofTree.premisses[1])})`;
        case 'AndElimFst': return `fst (${generateCode(proofTree.premisses[0])})`;
        case 'AndElimSnd': return `snd (${generateCode(proofTree.premisses[0])})`;
        case 'OrIntroFst': return `inl (${generateCode(proofTree.premisses[0])})`;
        case 'OrIntroSnd': return `inr (${generateCode(proofTree.premisses[0])})`;
        case 'OrElim': return `case ${generateCode(proofTree.premisses[0])} of inl ${rule.value[0]} => ${generateCode(proofTree.premisses[1])}, inr ${rule.value[1]} => ${generateCode(proofTree.premisses[2])}`;
        case 'ImplIntro': return `fn ${rule.value} => ${generateCode(proofTree.premisses[0])}`;
        case 'ImplElim': return `(${generateCode(proofTree.premisses[0])}) (${generateCode(proofTree.premisses[1])})`;
        case 'FalsumElim': return `abort (${generateCode(proofTree.premisses[0])})`;
        case 'ForAllIntro': return `fn ${rule.value} => ${generateCode(proofTree.premisses[0])}`;
        case 'ForAllElim': return `(${generateCode(proofTree.premisses[0])}) (${generateCode(proofTree.premisses[1])})`;
        case 'ExistsIntro': return `(${generateCode(proofTree.premisses[0])}, ${generateCode(proofTree.premisses[1])})`;
        case 'ExistsElim': return `let (${rule.value[0]}, ${rule.value[1]}) = ${generateCode(proofTree.premisses[0])} in ${generateCode(proofTree.premisses[1])}`;
        case 'Sorry': return 'sorry';
    }
};