import React from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { initI18n, syncLanguage } from '../../i18n';
import { CodexAccountsContent } from '../../pages/CodexAccountsContent';
import './style.css';

type CodexRemoteHostApi = {
  platformId: 'codex';
  packageVersion?: string | null;
  locale?: string | null;
  theme?: string | null;
};

const roots = new WeakMap<HTMLElement, Root>();

function normalizeTheme(theme: string | null | undefined): string {
  return theme && theme.trim() ? theme : document.documentElement.dataset.theme || 'dark';
}

function normalizeLocale(locale: string | null | undefined): string {
  return locale && locale.trim() ? locale : 'zh-CN';
}

function CodexRemoteApp() {
  return (
    <React.StrictMode>
      <div className="codex-platform-ui-root">
        <CodexAccountsContent />
      </div>
    </React.StrictMode>
  );
}

export async function mount(container: HTMLElement, hostApi: CodexRemoteHostApi) {
  unmount(container);

  const theme = normalizeTheme(hostApi.theme);
  const locale = normalizeLocale(hostApi.locale);
  document.documentElement.dataset.theme = theme;
  document.documentElement.lang = locale;

  await initI18n();
  await syncLanguage(locale);

  const root = createRoot(container);
  roots.set(container, root);
  root.render(<CodexRemoteApp />);

  return () => unmount(container);
}

export function unmount(container: HTMLElement) {
  const root = roots.get(container);
  if (!root) return;
  root.unmount();
  roots.delete(container);
}
