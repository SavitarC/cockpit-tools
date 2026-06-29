import { useEffect, useMemo } from 'react';
import { PlatformPackageToolbar } from '../components/PlatformPackageToolbar';
import { PlatformPackageUnavailablePage } from '../components/PlatformPackageUnavailablePage';
import { PlatformOverviewTabsHeader } from '../components/platform/PlatformOverviewTabsHeader';
import { PlatformRuntimePageHost } from '../components/platform/PlatformRuntimePageHost';
import {
  getPlatformPackageFromPackages,
  usePlatformPackageStore,
} from '../stores/usePlatformPackageStore';

const WORKBUDDY_REMOTE_TABS_SLOT_ID = 'workbuddy-remote-tabs-slot';

export function WorkbuddyAccountsPage() {
  const packages = usePlatformPackageStore((state) => state.packages);
  const initialized = usePlatformPackageStore((state) => state.initialized);
  const refreshPlatformPackages = usePlatformPackageStore((state) => state.refresh);
  const platformPackage = useMemo(
    () => getPlatformPackageFromPackages(packages, 'workbuddy'),
    [packages],
  );

  useEffect(() => {
    if (initialized) {
      return;
    }
    void refreshPlatformPackages().catch((error) => {
      console.error('Failed to refresh WorkBuddy platform package:', error);
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
    <div className="ghcp-accounts-page workbuddy-accounts-page">
      <PlatformOverviewTabsHeader
        platform="workbuddy"
        active="overview"
        hideTabs
        remoteTabsSlotId={runtimeReady ? WORKBUDDY_REMOTE_TABS_SLOT_ID : undefined}
        rightSlot={<PlatformPackageToolbar platformId="workbuddy" />}
      />

      {!runtimeReady ? (
        <PlatformPackageUnavailablePage platformId="workbuddy" state={platformPackage} />
      ) : platformPackage ? (
        <PlatformRuntimePageHost
          platformId="workbuddy"
          state={platformPackage}
          tabsSlotId={WORKBUDDY_REMOTE_TABS_SLOT_ID}
        />
      ) : (
        <PlatformPackageUnavailablePage platformId="workbuddy" state={platformPackage} />
      )}
    </div>
  );
}
