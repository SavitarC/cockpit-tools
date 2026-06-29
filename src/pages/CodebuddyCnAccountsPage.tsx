import { useEffect, useMemo } from 'react';
import { PlatformPackageToolbar } from '../components/PlatformPackageToolbar';
import { PlatformPackageUnavailablePage } from '../components/PlatformPackageUnavailablePage';
import { PlatformOverviewTabsHeader } from '../components/platform/PlatformOverviewTabsHeader';
import { PlatformRuntimePageHost } from '../components/platform/PlatformRuntimePageHost';
import {
  getPlatformPackageFromPackages,
  usePlatformPackageStore,
} from '../stores/usePlatformPackageStore';

const CODEBUDDY_CN_REMOTE_TABS_SLOT_ID = 'codebuddy-cn-remote-tabs-slot';

export function CodebuddyCnAccountsPage() {
  const packages = usePlatformPackageStore((state) => state.packages);
  const initialized = usePlatformPackageStore((state) => state.initialized);
  const refreshPlatformPackages = usePlatformPackageStore((state) => state.refresh);
  const platformPackage = useMemo(
    () => getPlatformPackageFromPackages(packages, 'codebuddy_cn'),
    [packages],
  );

  useEffect(() => {
    if (initialized) {
      return;
    }
    void refreshPlatformPackages().catch((error) => {
      console.error('Failed to refresh CodeBuddy CN platform package:', error);
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
        platform="codebuddy_cn"
        active="overview"
        hideTabs
        remoteTabsSlotId={runtimeReady ? CODEBUDDY_CN_REMOTE_TABS_SLOT_ID : undefined}
        rightSlot={<PlatformPackageToolbar platformId="codebuddy_cn" />}
      />

      {!runtimeReady ? (
        <PlatformPackageUnavailablePage platformId="codebuddy_cn" state={platformPackage} />
      ) : platformPackage ? (
        <PlatformRuntimePageHost
          platformId="codebuddy_cn"
          state={platformPackage}
          tabsSlotId={CODEBUDDY_CN_REMOTE_TABS_SLOT_ID}
        />
      ) : (
        <PlatformPackageUnavailablePage platformId="codebuddy_cn" state={platformPackage} />
      )}
    </div>
  );
}
