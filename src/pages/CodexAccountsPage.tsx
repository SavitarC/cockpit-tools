import { useEffect, useMemo } from 'react';
import { PlatformPackageToolbar } from '../components/PlatformPackageToolbar';
import { PlatformPackageUnavailablePage } from '../components/PlatformPackageUnavailablePage';
import { PlatformOverviewTabsHeader } from '../components/platform/PlatformOverviewTabsHeader';
import { PlatformRuntimePageHost } from '../components/platform/PlatformRuntimePageHost';
import {
  getPlatformPackageFromPackages,
  usePlatformPackageStore,
} from '../stores/usePlatformPackageStore';

export function CodexAccountsPage() {
  const packages = usePlatformPackageStore((state) => state.packages);
  const initialized = usePlatformPackageStore((state) => state.initialized);
  const refreshPlatformPackages = usePlatformPackageStore((state) => state.refresh);
  const platformPackage = useMemo(
    () => getPlatformPackageFromPackages(packages, 'codex'),
    [packages],
  );

  useEffect(() => {
    if (initialized) return;
    void refreshPlatformPackages().catch((error) => {
      console.error('Failed to refresh Codex platform package:', error);
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
    <div className="codex-platform-package-page">
      {runtimeReady ? (
        <div className="codex-platform-package-floating-toolbar">
          <PlatformPackageToolbar platformId="codex" />
        </div>
      ) : (
        <PlatformOverviewTabsHeader
          platform="codex"
          active="overview"
          hideTabs
          rightSlot={<PlatformPackageToolbar platformId="codex" />}
        />
      )}

      {!runtimeReady ? (
        <PlatformPackageUnavailablePage platformId="codex" state={platformPackage} />
      ) : platformPackage ? (
        <PlatformRuntimePageHost platformId="codex" state={platformPackage} />
      ) : (
        <PlatformPackageUnavailablePage platformId="codex" state={platformPackage} />
      )}
    </div>
  );
}
