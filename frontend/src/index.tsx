import 'katex/dist/katex.css'
import '../style/index.css'
import React from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './domain/app/components/app';
import {initialize} from 'alice';

initialize();

const container = document.getElementById('app');

if (!container) {
    throw new Error('app container not found.');
}

// Render App Component
const root = createRoot(container);
root.render(<App />);