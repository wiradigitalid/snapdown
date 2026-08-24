import React, { useEffect } from 'react';
import ReactDOM from 'react-dom/client';
import { App } from './App';
import { CaptureOverlay } from './components/CaptureOverlay';
import './styles/tokens.css';

export const Root: React.FC = () => {
  const isOverlay = new URLSearchParams(window.location.search).get('overlay') === 'true';

  useEffect(() => {
    if (isOverlay) {
      document.documentElement.classList.add('overlay-mode');
      document.body.classList.add('overlay-mode');
    } else {
      document.documentElement.classList.remove('overlay-mode');
      document.body.classList.remove('overlay-mode');
    }
  }, [isOverlay]);

  return isOverlay ? <CaptureOverlay /> : <App />;
};

const rootElement = document.getElementById('root');
if (rootElement) {
  ReactDOM.createRoot(rootElement).render(
    <React.StrictMode>
      <Root />
    </React.StrictMode>
  );
}

