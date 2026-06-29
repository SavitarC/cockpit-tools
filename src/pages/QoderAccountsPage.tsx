import { useEffect, useMemo } from 'react';
import { PlatformPackageToolbar } from '../components/PlatformPackageToolbar';
import { PlatformPackageUnavailablePage } from '../components/PlatformPackageUnavailablePage';
import { PlatformOverviewTabsHeader } from '../components/platform/PlatformOverviewTabsHeader';
import { PlatformRuntimePageHost } from '../components/platform/PlatformRuntimePageHost';
import {
  getPlatformPackageFromPackages,
  usePlatformPackageStore,
} from '../stores/usePlatformPackageStore';

const QODER_REMOTE_TABS_SLOT_ID = 'qoder-remote-tabs-slot';

export function QoderAccountsPage() {
  const packages = usePlatformPackageStore((state) => state.packages);
  const initialized = usePlatformPackageStore((state) => state.initialized);
  const refreshPlatformPackages = usePlatformPackageStore((state) => state.refresh);
  const platformPackage = useMemo(
    () => getPlatformPackageFromPackages(packages, 'qoder'),
    [packages],
  );

  useEffect(() => {
    if (initialized) {
      return;
    }
    void refreshPlatformPackages().catch((error) => {
      console.error('Failed to refresh Qoder platform package:', error);
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
    <div className="ghcp-accounts-page qoder-accounts-page">
      <PlatformOverviewTabsHeader
        platform="qoder"
        active="overview"
        hideTabs
        remoteTabsSlotId={runtimeReady ? QODER_REMOTE_TABS_SLOT_ID : undefined}
        rightSlot={<PlatformPackageToolbar platformId="qoder" />}
      />

      {!runtimeReady ? (
        <PlatformPackageUnavailablePage platformId="qoder" state={platformPackage} />
      ) : platformPackage ? (
        <PlatformRuntimePageHost
          platformId="qoder"
          state={platformPackage}
          tabsSlotId={QODER_REMOTE_TABS_SLOT_ID}
        />
      ) : (
        <PlatformPackageUnavailablePage platformId="qoder" state={platformPackage} />
      )}
    </div>
  );
}
