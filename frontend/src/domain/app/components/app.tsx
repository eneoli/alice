import React, { useState } from 'react';
import { Header } from './header';
import { CodeEditor } from '../../code-editor/components/code-editor';
import { VisualProofEditor } from '../../visual-proof-editor/components/visual-proof-editor';
import { ConfigProvider, Drawer, message, theme as antdTheme, ThemeConfig } from 'antd';
import { Prop, export_as_ocaml, generate_proof_term_from_proof_tree, parse_prop, verify } from 'alice';
import { debounce, isEqual } from 'lodash';
import { CodeModal } from './code-modal';
import { VisualProofEditorProofTree, visualProofEditorProofTreeIntoAliceProofTree } from '../../visual-proof-editor/lib/visual-proof-editor-proof-tree';
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
        try {
            const code = generate_proof_term_from_proof_tree(visualProofEditorProofTreeIntoAliceProofTree(proofTree));
            console.log(code);
            setProofTerm(code);
        } catch(_) {
            console.error('Generation failed');
        }
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
