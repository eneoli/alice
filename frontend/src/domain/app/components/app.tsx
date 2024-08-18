import React, { useState } from 'react';
import { Header } from './header';
import { CodeEditor } from '../../code-editor/components/code-editor';
import { VisualProofEditor } from '../../visual-proof-editor/components/visual-proof-editor';
import { ConfigProvider, message, theme as antdTheme, ThemeConfig } from 'antd';
import { Prop, VerificationResult, export_as_ocaml, generate_proof_term_from_proof_tree, parse_prop, print_prop, verify } from 'alice';
import { debounce, isEqual } from 'lodash';
import { CodeModal } from './code-modal';
import { VisualProofEditorProofTree, visualProofEditorProofTreeIntoAliceProofTree } from '../../visual-proof-editor/lib/visual-proof-editor-proof-tree';
import { MathJax3Config, MathJaxContext } from 'better-react-mathjax';
import mathjax from 'mathjax/es5/tex-svg';
import bussproofs from 'mathjax/es5/input/tex/extensions/bussproofs'
import { Tutor } from '../../tutor/components/tutor';
import { css } from '@emotion/css';

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
    const [verificationResult, setVerificationResult] = useState<VerificationResult | null>(null);
    const [_messageApi, contextHolder] = message.useMessage();

    const handlePropChange = debounce((propString: string) => {
        try {
            const newProp = parse_prop(propString);

            if (!isEqual(prop, newProp)) {
                setProp(newProp);
                setProofTerm('sorry');

                const verificationResult = verify(propString, 'sorry');
                setVerificationResult(verificationResult);
            }
        } catch (e) {
            setProp(null);
            console.error(e);
        }
    }, 500);

    const handleProofTreeChange = (proofTree: VisualProofEditorProofTree) => {
        try {
            const code = generate_proof_term_from_proof_tree(visualProofEditorProofTreeIntoAliceProofTree(proofTree));
            setProofTerm(code);

            if (prop) {
                const result = verify(print_prop(prop), code);
                setVerificationResult(result);
            }
        } catch (e) {
            console.error(e);
        }
    };

    const handleVerify = (prop: string) => {
        let isProof = false;
        let allGoalsClosed = false;
        try {
            const result = verify(prop, proofTerm);
            isProof = result.kind === 'TypeCheckSucceeded';
            allGoalsClosed = result.kind === 'TypeCheckSucceeded' && result.value.result.goals.length === 0;
            setVerificationResult(result);
        } catch (e: unknown) {
            console.error(e);
        }

        if (isProof && allGoalsClosed) {
            message.success('Your proof is correct. Well done!');
        }

        if (isProof && !allGoalsClosed) {
            message.info('You still have open goals.');
        }

        if (!isProof) {
            message.error('Your proof contains errors.');
        }
    };

    const handleOcamlExport = () => {
        if (!prop) {
            return;
        }

        setShowCodeExport(true);
    };

    const handleVisualEditorReset = () => {
        setProofTerm('sorry');

        if (prop) {
            setVerificationResult(verify(print_prop(prop), 'sorry'));
        }
    };

    return (
        <ConfigProvider theme={theme}>
            <MathJaxContext
                src={mathjax}
                config={mathjaxConfig}
                version={3}>
                {contextHolder}
                <div className={cssAppContainer}>
                    <Header
                        onPropChange={handlePropChange}
                        onVerify={handleVerify}
                        onExportAsOcaml={handleOcamlExport}
                        enableTutor={!!prop}
                        onTutorClick={() => setShowTutor(!showTutor)}
                    />

                    <div className={cssBodyContainer}>
                        <div className={cssEditorContainer}>
                            {prop && (
                                <>
                                    <VisualProofEditor
                                        prop={prop}
                                        onProofTreeChange={handleProofTreeChange}
                                        onReset={handleVisualEditorReset}
                                    />

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
                        </div>
                        <div className={cssTutorContainer} style={{ marginRight: showTutor ? 0 : '-450px' }}>
                            {
                                verificationResult && (
                                    <Tutor
                                        code={proofTerm}
                                        verificationResult={verificationResult}
                                    />
                                )
                            }
                        </div>
                    </div>
                </div>
            </MathJaxContext>
        </ConfigProvider>
    );
}

const cssAppContainer = css`
    display: flex;
    flex-direction: column;
    height: 100%;
`;

const cssBodyContainer = css`
    display: flex;
    flex: 1;
    flex-direction: row;
    overflow-x: hidden;
`;

const cssEditorContainer = css`
    flex: 1 1 auto;
    flex-wrap: wrap;
    overflow: auto;
`;

const cssTutorContainer = css`
    box-sizing: border-box;
    flex: 0 0 450px;
    background: linear-gradient(90deg, rgba(46,77,97,1) 0%, rgba(43,63,89,1) 100%);
    width: 450px;
    height: 100%;
    padding: 25px;
    transition: width, 0.3s ease-in-out;
    color: #fefefe;
`;

const theme: ThemeConfig = {
    algorithm: antdTheme.darkAlgorithm,
    token: {
        colorPrimary: '#006af5',
        colorBgBase: '#233348',
        colorPrimaryBg: 'transparent',
    },
};
