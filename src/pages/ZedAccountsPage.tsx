import { useEffect, useMemo } from 'react';
import { PlatformPackageToolbar } from '../components/PlatformPackageToolbar';
import { PlatformPackageUnavailablePage } from '../components/PlatformPackageUnavailablePage';
import { PlatformRuntimePageHost } from '../components/platform/PlatformRuntimePageHost';
import { ZedOverviewTabsHeader } from '../components/ZedOverviewTabsHeader';
import {
  getPlatformPackageFromPackages,
  usePlatformPackageStore,
} from '../stores/usePlatformPackageStore';

const ZED_REMOTE_TABS_SLOT_ID = 'zed-remote-tabs-slot';

export function ZedAccountsPage() {
  const packages = usePlatformPackageStore((state) => state.packages);
  const initialized = usePlatformPackageStore((state) => state.initialized);
  const refreshPlatformPackages = usePlatformPackageStore((state) => state.refresh);
  const platformPackage = useMemo(
    () => getPlatformPackageFromPackages(packages, 'zed'),
    [packages],
  );

  useEffect(() => {
    if (initialized) {
      return;
    }
    void refreshPlatformPackages().catch((error) => {
      console.error('Failed to refresh Zed platform package:', error);
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
    <div className="ghcp-accounts-page zed-accounts-page">
      <ZedOverviewTabsHeader
        hideTabs
        remoteTabsSlotId={runtimeReady ? ZED_REMOTE_TABS_SLOT_ID : undefined}
        rightSlot={<PlatformPackageToolbar platformId="zed" />}
      />

      {!runtimeReady ? (
        <PlatformPackageUnavailablePage platformId="zed" state={platformPackage} />
      ) : platformPackage ? (
        <PlatformRuntimePageHost
          platformId="zed"
          state={platformPackage}
          tabsSlotId={ZED_REMOTE_TABS_SLOT_ID}
        />
      ) : (
        <PlatformPackageUnavailablePage platformId="zed" state={platformPackage} />
      )}
    </div>
  );
}
