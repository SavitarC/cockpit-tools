import React, { useMemo, useState } from 'react';
import { createPortal } from 'react-dom';
import { createRoot, type Root } from 'react-dom/client';
import { Layers } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { initI18n, syncLanguage } from '../../i18n';
import { WindsurfIcon } from '../../components/icons/WindsurfIcon';
import { WindsurfAccountsContent, type WindsurfAccountsContentTab } from '../../pages/WindsurfAccountsContent';
import './style.css';

type WindsurfRemoteHostApi = {
  platformId: 'windsurf';
  packageVersion?: string | null;
  locale?: string | null;
  theme?: string | null;
  tabsSlotId?: string | null;
};

const roots = new WeakMap<HTMLElement, Root>();

function normalizeTheme(theme: string | null | undefined): string {
  return theme && theme.trim() ? theme : document.documentElement.dataset.theme || 'dark';
}

function normalizeLocale(locale: string | null | undefined): string {
  return locale && locale.trim() ? locale : 'zh-CN';
}

function WindsurfRemoteTabs({
  activeTab,
  onTabChange,
}: {
  activeTab: WindsurfAccountsContentTab;
  onTabChange: (tab: WindsurfAccountsContentTab) => void;
}) {
  const { t } = useTranslation();
  const tabs = useMemo(
    () => [
      {
        key: 'overview' as const,
        label: t('overview.title', '账号总览'),
        icon: <WindsurfIcon className="tab-icon" />,
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
          className={`filter-tab${activeTab === tab.key ? ' active' : ''}`}
          aria-current={activeTab === tab.key ? 'page' : undefined}
          onClick={() => onTabChange(tab.key)}
        >
          {tab.icon}
          <span>{tab.label}</span>
        </button>
      ))}
    </>
  );
}

function WindsurfRemoteApp({
  tabsContainer,
}: {
  tabsContainer: HTMLElement | null;
}) {
  const [activeTab, setActiveTab] = useState<WindsurfAccountsContentTab>('overview');

  return (
    <React.StrictMode>
      {tabsContainer
        ? createPortal(
            <WindsurfRemoteTabs activeTab={activeTab} onTabChange={setActiveTab} />,
            tabsContainer,
          )
        : null}
      <WindsurfAccountsContent activeTab={activeTab} />
    </React.StrictMode>
  );
}

export async function mount(container: HTMLElement, hostApi: WindsurfRemoteHostApi) {
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
  root.render(<WindsurfRemoteApp tabsContainer={tabsContainer} />);

  return () => unmount(container);
}

export function unmount(container: HTMLElement) {
  const root = roots.get(container);
  if (!root) return;
  root.unmount();
  roots.delete(container);
}
