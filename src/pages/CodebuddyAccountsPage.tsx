import { useEffect, useMemo } from 'react';
import { PlatformPackageToolbar } from '../components/PlatformPackageToolbar';
import { PlatformPackageUnavailablePage } from '../components/PlatformPackageUnavailablePage';
import { PlatformOverviewTabsHeader } from '../components/platform/PlatformOverviewTabsHeader';
import { PlatformRuntimePageHost } from '../components/platform/PlatformRuntimePageHost';
import {
  getPlatformPackageFromPackages,
  usePlatformPackageStore,
} from '../stores/usePlatformPackageStore';

const CODEBUDDY_REMOTE_TABS_SLOT_ID = 'codebuddy-remote-tabs-slot';

export function CodebuddyAccountsPage() {
  const packages = usePlatformPackageStore((state) => state.packages);
  const initialized = usePlatformPackageStore((state) => state.initialized);
  const refreshPlatformPackages = usePlatformPackageStore((state) => state.refresh);
  const platformPackage = useMemo(
    () => getPlatformPackageFromPackages(packages, 'codebuddy'),
    [packages],
  );

  useEffect(() => {
    if (initialized) {
      return;
    }
    void refreshPlatformPackages().catch((error) => {
      console.error('Failed to refresh CodeBuddy platform package:', error);
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
    <div className="ghcp-accounts-page codebuddy-accounts-page">
      <PlatformOverviewTabsHeader
        platform="codebuddy"
        active="overview"
        hideTabs
        remoteTabsSlotId={runtimeReady ? CODEBUDDY_REMOTE_TABS_SLOT_ID : undefined}
        rightSlot={<PlatformPackageToolbar platformId="codebuddy" />}
      />

      {!runtimeReady ? (
        <PlatformPackageUnavailablePage platformId="codebuddy" state={platformPackage} />
      ) : platformPackage ? (
        <PlatformRuntimePageHost
          platformId="codebuddy"
          state={platformPackage}
          tabsSlotId={CODEBUDDY_REMOTE_TABS_SLOT_ID}
        />
      ) : (
        <PlatformPackageUnavailablePage platformId="codebuddy" state={platformPackage} />
      )}
    </div>
  );
}
