import { useEffect, useMemo } from 'react';
import { PlatformPackageToolbar } from '../components/PlatformPackageToolbar';
import { PlatformPackageUnavailablePage } from '../components/PlatformPackageUnavailablePage';
import { GeminiOverviewTabsHeader } from '../components/GeminiOverviewTabsHeader';
import { PlatformRuntimePageHost } from '../components/platform/PlatformRuntimePageHost';
import {
  getPlatformPackageFromPackages,
  usePlatformPackageStore,
} from '../stores/usePlatformPackageStore';

const GEMINI_REMOTE_TABS_SLOT_ID = 'gemini-remote-tabs-slot';

export function GeminiAccountsPage() {
  const packages = usePlatformPackageStore((state) => state.packages);
  const initialized = usePlatformPackageStore((state) => state.initialized);
  const refreshPlatformPackages = usePlatformPackageStore((state) => state.refresh);
  const platformPackage = useMemo(
    () => getPlatformPackageFromPackages(packages, 'gemini'),
    [packages],
  );

  useEffect(() => {
    if (initialized) {
      return;
    }
    void refreshPlatformPackages().catch((error) => {
      console.error('Failed to refresh Gemini platform package:', error);
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
    <div className="ghcp-accounts-page gemini-accounts-page">
      <GeminiOverviewTabsHeader
        active="overview"
        hideTabs
        remoteTabsSlotId={runtimeReady ? GEMINI_REMOTE_TABS_SLOT_ID : undefined}
        rightSlot={<PlatformPackageToolbar platformId="gemini" />}
      />

      {!runtimeReady ? (
        <PlatformPackageUnavailablePage platformId="gemini" state={platformPackage} />
      ) : platformPackage ? (
        <PlatformRuntimePageHost
          platformId="gemini"
          state={platformPackage}
          tabsSlotId={GEMINI_REMOTE_TABS_SLOT_ID}
        />
      ) : (
        <PlatformPackageUnavailablePage platformId="gemini" state={platformPackage} />
      )}
    </div>
  );
}
