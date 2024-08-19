import React, { useCallback, useState } from 'react';
import { Header } from './header';
import { CodeEditor } from '../../code-editor/components/code-editor';
import { VisualProofEditor } from '../../visual-proof-editor/components/visual-proof-editor';
import { ConfigProvider, message, theme as antdTheme, ThemeConfig } from 'antd';
import { Prop, VerificationResult, export_as_ocaml, generate_proof_term_from_proof_tree, parse_prop, verify } from 'alice';
import { debounce, isEqual } from 'lodash';
import { CodeModal } from './code-modal';
import { aliceProofTreeIntoVisualProofEditorProofTree, VisualProofEditorProofTree, visualProofEditorProofTreeIntoAliceProofTree } from '../../visual-proof-editor/lib/visual-proof-editor-proof-tree';
import { MathJax3Config, MathJaxContext } from 'better-react-mathjax';
import mathjax from 'mathjax/es5/tex-svg';
import bussproofs from 'mathjax/es5/input/tex/extensions/bussproofs'
import { Tutor } from '../../tutor/components/tutor';
import { css } from '@emotion/css';
import { AssumptionContext } from '../../visual-proof-editor/proof-rule';
import { v4 } from 'uuid';
import { VisualProofEditorReasoningContext } from '../../visual-proof-editor/lib/visual-proof-editor-reasoning-context';

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

    const [initialPrimaryContext, setInitialPrimaryContext] = useState<VisualProofEditorReasoningContext | null>(null);
    const [initialAssumptions, setInitialAssumptions] = useState<AssumptionContext[]>([]);

    const handlePropChange = debounce((propString: string) => {
        try {
            const newProp = parse_prop(propString);

            if (isEqual(prop, newProp)) {
                return;
            }

            setProp(newProp);
            setProofTerm('sorry');

            const verificationResult = verify(newProp, 'sorry');
            setVerificationResult(verificationResult);

            const proofTree: VisualProofEditorProofTree = {
                id: v4(),
                premisses: [],
                rule: null,
                conclusion: { kind: 'PropIsTrue', value: newProp },
            };

            const primaryCtx: VisualProofEditorReasoningContext = {
                id: v4(),
                selectedNodeId: null,
                isDragging: false,
                x: 0,
                y: 0,
                proofTree: proofTree,
            };

            setInitialPrimaryContext(primaryCtx);
            setInitialAssumptions([]);
        } catch (e) {
            setProp(null);
            console.error(e);
        }
    }, 500);

    const handleProofTermChange = useCallback(debounce((newProofTerm: string) => {

        if (newProofTerm.trim() === proofTerm.trim()) {
            return;
        }

        setProofTerm(newProofTerm);

        if (!prop) {
            return;
        }

        const verificationResult = verify(prop, newProofTerm);
        setVerificationResult(verificationResult);

        if (verificationResult.kind === 'TypeCheckSucceeded') {
            const primaryReasoningCtxId = v4();
            const proofTree = verificationResult.value.result.proof_tree;
            const proofTreeResult = aliceProofTreeIntoVisualProofEditorProofTree(primaryReasoningCtxId, proofTree);

            const primaryCtx: VisualProofEditorReasoningContext = {
                id: primaryReasoningCtxId,
                selectedNodeId: null,
                isDragging: false,
                x: 0,
                y: 0,
                proofTree: proofTreeResult.proofTree
            };

            setInitialPrimaryContext(primaryCtx);
            setInitialAssumptions(proofTreeResult.assumptions);
        }
    }, 500), [prop]);

    const handleProofTreeChange = useCallback((proofTree: VisualProofEditorProofTree) => {
        const code = generate_proof_term_from_proof_tree(
            visualProofEditorProofTreeIntoAliceProofTree(proofTree)
        );
        setProofTerm(code);

        if (prop) {
            const result = verify(prop, code);
            setVerificationResult(result);
        }
    }, [prop]);

    const handleVerify = useCallback((prop: string) => {
        const result = verify(parse_prop(prop), proofTerm);
        setVerificationResult(result);

        if (
            result.kind === 'LexerError' ||
            result.kind === 'ParserError' ||
            result.kind === 'ProofPipelineError'
        ) {
            message.error('Your input is malformed. Check the Tutor for more information.');
            return;
        }

        const isProof = result.kind === 'TypeCheckSucceeded';
        const allGoalsClosed = result.kind === 'TypeCheckSucceeded' && result.value.result.goals.length === 0;

        if (isProof && allGoalsClosed) {
            message.success('Your proof is correct. Well done!');
        }

        if (isProof && !allGoalsClosed) {
            message.info('You still have open goals.');
        }

        if (!isProof) {
            message.error('Your proof contains errors.');
        }
    }, [proofTerm, message]);

    const handleOcamlExport = useCallback(() => {
        if (!prop) {
            return;
        }

        setShowCodeExport(true);
    }, [prop]);

    const handleVisualEditorReset = useCallback(() => {
        setProofTerm('sorry');

        if (prop) {
            setVerificationResult(verify(prop, 'sorry'));
        }
    }, [prop]);

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
                            {!(prop && initialPrimaryContext) && (
                                <div style={{ textAlign: 'center', color: '#192434' }}>
                                    <h1>Alice is ready.</h1>
                                    <h2>Please enter a proposition to begin.</h2>
                                </div>
                            )}

                            {prop && initialPrimaryContext && (
                                <>
                                    <VisualProofEditor
                                        prop={prop}
                                        initialPrimaryReasoningContext={initialPrimaryContext}
                                        initialAssumptions={initialAssumptions}
                                        onProofTreeChange={handleProofTreeChange}
                                        onReset={handleVisualEditorReset}
                                    />

                                    <div style={{ marginTop: 20 }}>
                                        <CodeEditor
                                            height={'20vh'}
                                            initialValue={proofTerm}
                                            onChange={handleProofTermChange}
                                        />
                                    </div>
                                </>
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
