import { useEffect, useMemo } from 'react';
import { PlatformPackageToolbar } from '../components/PlatformPackageToolbar';
import { PlatformPackageUnavailablePage } from '../components/PlatformPackageUnavailablePage';
import { PlatformRuntimePageHost } from '../components/platform/PlatformRuntimePageHost';
import { KiroOverviewTabsHeader } from '../components/KiroOverviewTabsHeader';
import {
  getPlatformPackageFromPackages,
  usePlatformPackageStore,
} from '../stores/usePlatformPackageStore';

const KIRO_REMOTE_TABS_SLOT_ID = 'kiro-remote-tabs-slot';

export function KiroAccountsPage() {
  const packages = usePlatformPackageStore((state) => state.packages);
  const initialized = usePlatformPackageStore((state) => state.initialized);
  const refreshPlatformPackages = usePlatformPackageStore((state) => state.refresh);
  const platformPackage = useMemo(
    () => getPlatformPackageFromPackages(packages, 'kiro'),
    [packages],
  );

  useEffect(() => {
    if (initialized) {
      return;
    }
    void refreshPlatformPackages().catch((error) => {
      console.error('Failed to refresh Kiro platform package:', error);
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
    <div className="ghcp-accounts-page kiro-accounts-page">
      <KiroOverviewTabsHeader
        hideTabs
        remoteTabsSlotId={runtimeReady ? KIRO_REMOTE_TABS_SLOT_ID : undefined}
        rightSlot={<PlatformPackageToolbar platformId="kiro" />}
      />

      {!runtimeReady ? (
        <PlatformPackageUnavailablePage platformId="kiro" state={platformPackage} />
      ) : platformPackage ? (
        <PlatformRuntimePageHost
          platformId="kiro"
          state={platformPackage}
          tabsSlotId={KIRO_REMOTE_TABS_SLOT_ID}
        />
      ) : (
        <PlatformPackageUnavailablePage platformId="kiro" state={platformPackage} />
      )}
    </div>
  );
}
