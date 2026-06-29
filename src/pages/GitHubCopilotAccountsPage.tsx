import { useEffect, useMemo } from 'react';
import { GitHubCopilotOverviewTabsHeader } from '../components/GitHubCopilotOverviewTabsHeader';
import { PlatformPackageToolbar } from '../components/PlatformPackageToolbar';
import { PlatformPackageUnavailablePage } from '../components/PlatformPackageUnavailablePage';
import { PlatformRuntimePageHost } from '../components/platform/PlatformRuntimePageHost';
import {
  getPlatformPackageFromPackages,
  usePlatformPackageStore,
} from '../stores/usePlatformPackageStore';

const GITHUB_COPILOT_REMOTE_TABS_SLOT_ID = 'github-copilot-remote-tabs-slot';

export function GitHubCopilotAccountsPage() {
  const packages = usePlatformPackageStore((state) => state.packages);
  const initialized = usePlatformPackageStore((state) => state.initialized);
  const refreshPlatformPackages = usePlatformPackageStore((state) => state.refresh);
  const platformPackage = useMemo(
    () => getPlatformPackageFromPackages(packages, 'github-copilot'),
    [packages],
  );

  useEffect(() => {
    if (initialized) {
      return;
    }
    void refreshPlatformPackages().catch((error) => {
      console.error('Failed to refresh GitHub Copilot platform package:', error);
    });
  }, [initialized, refreshPlatformPackages]);

  const runtimeReady = Boolean(
    platformPackage
    && platformPackage.packageMode === 'hotUpdate'
    && platformPackage.runtimeReady
    && (
      platformPackage.installStatus === 'installed'
      || platformPackage.installStatus === 'updateAvailable'
    ),
  );

  return (
    <div className="ghcp-accounts-page github-copilot-accounts-page">
      <GitHubCopilotOverviewTabsHeader
        active="overview"
        hideTabs
        remoteTabsSlotId={runtimeReady ? GITHUB_COPILOT_REMOTE_TABS_SLOT_ID : undefined}
        rightSlot={<PlatformPackageToolbar platformId="github-copilot" />}
      />

      {!runtimeReady ? (
        <PlatformPackageUnavailablePage platformId="github-copilot" state={platformPackage} />
      ) : platformPackage ? (
        <PlatformRuntimePageHost
          platformId="github-copilot"
          state={platformPackage}
          tabsSlotId={GITHUB_COPILOT_REMOTE_TABS_SLOT_ID}
        />
      ) : (
        <PlatformPackageUnavailablePage platformId="github-copilot" state={platformPackage} />
      )}
    </div>
  );
}
