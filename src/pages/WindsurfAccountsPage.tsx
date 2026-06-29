import { useEffect, useMemo } from 'react';
import { PlatformPackageToolbar } from '../components/PlatformPackageToolbar';
import { PlatformPackageUnavailablePage } from '../components/PlatformPackageUnavailablePage';
import { WindsurfOverviewTabsHeader } from '../components/WindsurfOverviewTabsHeader';
import { PlatformRuntimePageHost } from '../components/platform/PlatformRuntimePageHost';
import {
  getPlatformPackageFromPackages,
  usePlatformPackageStore,
} from '../stores/usePlatformPackageStore';

const WINDSURF_REMOTE_TABS_SLOT_ID = 'windsurf-remote-tabs-slot';

export function WindsurfAccountsPage() {
  const packages = usePlatformPackageStore((state) => state.packages);
  const initialized = usePlatformPackageStore((state) => state.initialized);
  const refreshPlatformPackages = usePlatformPackageStore((state) => state.refresh);
  const platformPackage = useMemo(
    () => getPlatformPackageFromPackages(packages, 'windsurf'),
    [packages],
  );

  useEffect(() => {
    if (initialized) {
      return;
    }
    void refreshPlatformPackages().catch((error) => {
      console.error('Failed to refresh Windsurf platform package:', error);
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
    <div className="ghcp-accounts-page windsurf-accounts-page">
      <WindsurfOverviewTabsHeader
        active="overview"
        hideTabs
        remoteTabsSlotId={runtimeReady ? WINDSURF_REMOTE_TABS_SLOT_ID : undefined}
        rightSlot={<PlatformPackageToolbar platformId="windsurf" />}
      />

      {!runtimeReady ? (
        <PlatformPackageUnavailablePage platformId="windsurf" state={platformPackage} />
      ) : platformPackage ? (
        <PlatformRuntimePageHost
          platformId="windsurf"
          state={platformPackage}
          tabsSlotId={WINDSURF_REMOTE_TABS_SLOT_ID}
        />
      ) : (
        <PlatformPackageUnavailablePage platformId="windsurf" state={platformPackage} />
      )}
    </div>
  );
}
