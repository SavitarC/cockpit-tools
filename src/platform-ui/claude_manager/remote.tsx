import React, { useMemo, useState } from 'react';
import { createPortal } from 'react-dom';
import { createRoot, type Root } from 'react-dom/client';
import { Layers, Terminal } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { ClaudeIcon } from '../../components/icons/ClaudeIcon';
import { initI18n, syncLanguage } from '../../i18n';
import {
  ClaudeAccountsContent,
  type ClaudeAccountsContentSection,
} from '../../pages/ClaudeAccountsContent';
import './style.css';

type ClaudeRemoteHostApi = {
  platformId: 'claude_manager';
  packageVersion?: string | null;
  locale?: string | null;
  theme?: string | null;
  tabsSlotId?: string | null;
  runtimeParams?: Record<string, unknown>;
};

const roots = new WeakMap<HTMLElement, Root>();

function normalizeTheme(theme: string | null | undefined): string {
  return theme && theme.trim() ? theme : document.documentElement.dataset.theme || 'dark';
}

function normalizeLocale(locale: string | null | undefined): string {
  return locale && locale.trim() ? locale : 'zh-CN';
}

function resolveInitialSection(hostApi: ClaudeRemoteHostApi): ClaudeAccountsContentSection {
  return hostApi.runtimeParams?.initialSubPlatform === 'cli' ? 'cli' : 'desktop';
}

function ClaudeRemoteTabs({
  activeSection,
  onSectionChange,
}: {
  activeSection: ClaudeAccountsContentSection;
  onSectionChange: (section: ClaudeAccountsContentSection) => void;
}) {
  const { t } = useTranslation();
  const tabs = useMemo(
    () => [
      {
        key: 'desktop' as const,
        label: t('claude.subPlatform.desktop', 'Claude'),
        icon: <ClaudeIcon className="tab-icon" />,
      },
      {
        key: 'cli' as const,
        label: t('claude.subPlatform.cli', 'Claude CLI'),
        icon: <Terminal className="tab-icon" />,
      },
      {
        key: 'instances' as const,
        label: t('instances.title', '多开实例'),
        icon: <Layers className="tab-icon" />,
      },
    ],
    [t],
  );

  return (
    <>
      {tabs.map((tab) => (
        <button
          key={tab.key}
          type="button"
          className={`filter-tab${activeSection === tab.key ? ' active' : ''}`}
          aria-current={activeSection === tab.key ? 'page' : undefined}
          onClick={() => onSectionChange(tab.key)}
        >
          {tab.icon}
          <span>{tab.label}</span>
        </button>
      ))}
    </>
  );
}

function ClaudeRemoteApp({
  tabsContainer,
  initialSection,
}: {
  tabsContainer: HTMLElement | null;
  initialSection: ClaudeAccountsContentSection;
}) {
  const [activeSection, setActiveSection] = useState<ClaudeAccountsContentSection>(initialSection);

  return (
    <React.StrictMode>
      {tabsContainer
        ? createPortal(
            <ClaudeRemoteTabs activeSection={activeSection} onSectionChange={setActiveSection} />,
            tabsContainer,
          )
        : null}
      <div className="claude-manager-platform-ui-root">
        <ClaudeAccountsContent
          subPlatform={initialSection === 'cli' ? 'cli' : 'desktop'}
          activeSection={activeSection}
          onActiveSectionChange={setActiveSection}
        />
      </div>
    </React.StrictMode>
  );
}

export async function mount(container: HTMLElement, hostApi: ClaudeRemoteHostApi) {
  unmount(container);

  const theme = normalizeTheme(hostApi.theme);
  const locale = normalizeLocale(hostApi.locale);
  document.documentElement.dataset.theme = theme;
  document.documentElement.lang = locale;

  await initI18n();
  await syncLanguage(locale);

  const tabsContainer = hostApi.tabsSlotId
    ? document.getElementById(hostApi.tabsSlotId)
    : null;
  const root = createRoot(container);
  roots.set(container, root);
  root.render(
    <ClaudeRemoteApp
      tabsContainer={tabsContainer}
      initialSection={resolveInitialSection(hostApi)}
    />,
  );

  return () => unmount(container);
}

export function unmount(container: HTMLElement) {
  const root = roots.get(container);
  if (!root) return;
  root.unmount();
  roots.delete(container);
}
