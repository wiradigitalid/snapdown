import React, { useEffect } from 'react';
import ReactDOM from 'react-dom/client';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { App } from './App';
import { CaptureOverlay } from './components/CaptureOverlay';
import './styles/tokens.css';

function isOverlayMode(): boolean {
  try {
    const currentWindow = getCurrentWebviewWindow();
    if (currentWindow && currentWindow.label === 'overlay') {
      return true;
    }
  } catch {
    // Non-Tauri environment / test fallback
  }
  if (typeof window !== 'undefined') {
    const searchParams = new URLSearchParams(window.location.search);
    if (searchParams.get('overlay') === 'true') {
      return true;
    }
  }
  return false;
}

export const Root: React.FC = () => {
  const isOverlay = isOverlayMode();

  useEffect(() => {
    // Disable default browser context menu across application webview
    const handleContextMenu = (e: MouseEvent) => {
      // Allow context menu only on native editable text elements if needed, otherwise prevent default
      const target = e.target as HTMLElement | null;
      if (
        target &&
        (target.tagName === 'INPUT' ||
          target.tagName === 'TEXTAREA' ||
          target.isContentEditable)
      ) {
        return;
      }
      e.preventDefault();
    };

    window.addEventListener('contextmenu', handleContextMenu);
    return () => {
      window.removeEventListener('contextmenu', handleContextMenu);
    };
  }, []);

  useEffect(() => {
    // Synchronize active theme from localStorage
    const savedTheme = localStorage.getItem('snapdown-theme') || 'light';
    document.documentElement.setAttribute('data-theme', savedTheme);

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
