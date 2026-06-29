import { useEffect, useMemo } from 'react';
import { PlatformPackageToolbar } from '../components/PlatformPackageToolbar';
import { PlatformPackageUnavailablePage } from '../components/PlatformPackageUnavailablePage';
import { CursorOverviewTabsHeader } from '../components/CursorOverviewTabsHeader';
import { PlatformRuntimePageHost } from '../components/platform/PlatformRuntimePageHost';
import {
  getPlatformPackageFromPackages,
  usePlatformPackageStore,
} from '../stores/usePlatformPackageStore';

const CURSOR_REMOTE_TABS_SLOT_ID = 'cursor-remote-tabs-slot';

export function CursorAccountsPage() {
  const packages = usePlatformPackageStore((state) => state.packages);
  const initialized = usePlatformPackageStore((state) => state.initialized);
  const refreshPlatformPackages = usePlatformPackageStore((state) => state.refresh);
  const platformPackage = useMemo(
    () => getPlatformPackageFromPackages(packages, 'cursor'),
    [packages],
  );

  useEffect(() => {
    if (initialized) {
      return;
    }
    void refreshPlatformPackages().catch((error) => {
      console.error('Failed to refresh Cursor platform package:', error);
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
    <div className="ghcp-accounts-page cursor-accounts-page">
      <CursorOverviewTabsHeader
        active="overview"
        hideTabs
        remoteTabsSlotId={runtimeReady ? CURSOR_REMOTE_TABS_SLOT_ID : undefined}
        rightSlot={<PlatformPackageToolbar platformId="cursor" />}
      />

      {!runtimeReady ? (
        <PlatformPackageUnavailablePage platformId="cursor" state={platformPackage} />
      ) : platformPackage ? (
        <PlatformRuntimePageHost
          platformId="cursor"
          state={platformPackage}
          tabsSlotId={CURSOR_REMOTE_TABS_SLOT_ID}
        />
      ) : (
        <PlatformPackageUnavailablePage platformId="cursor" state={platformPackage} />
      )}
    </div>
  );
}
