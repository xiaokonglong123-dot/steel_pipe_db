// App entry point — React 19 StrictMode + root App component
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import App from './App';

// i18n: side-effect import triggers i18next initialization (detect language, load resources)
import './i18n';

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
