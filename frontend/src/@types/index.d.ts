declare module '*.png' {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const value: any;
    export = value;
}

declare module 'mathjax/es5/tex-svg' {
    const value: string;
    export = value;
}

declare module 'mathjax/es5/input/tex/extensions/bussproofs' {
    const value: string;
    export = value;
}