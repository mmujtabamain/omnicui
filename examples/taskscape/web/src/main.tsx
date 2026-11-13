// The web app's entry: the theme and reduced-motion preferences as the
// desktop app reads them, then the window.
import { initReducedMotion } from '@shared/lib/reducedMotion';
import { initTheme } from '@shared/lib/theme';
import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './app/App';
import './index.css';
import './material-symbols.css';

void initTheme();
void initReducedMotion();

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
