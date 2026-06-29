import { useEffect, useMemo } from 'react';
import { PlatformPackageToolbar } from '../components/PlatformPackageToolbar';
import { PlatformPackageUnavailablePage } from '../components/PlatformPackageUnavailablePage';
import { PlatformOverviewTabsHeader } from '../components/platform/PlatformOverviewTabsHeader';
import { PlatformRuntimePageHost } from '../components/platform/PlatformRuntimePageHost';
import {
  getPlatformPackageFromPackages,
  usePlatformPackageStore,
} from '../stores/usePlatformPackageStore';

type ClaudeSubPlatform = 'desktop' | 'cli';

interface ClaudeAccountsPageProps {
  subPlatform?: ClaudeSubPlatform;
}

const CLAUDE_PLATFORM_ID = 'claude_manager';
const CLAUDE_REMOTE_TABS_SLOT_ID = 'claude-manager-remote-tabs-slot';

export function ClaudeAccountsPage({ subPlatform = 'desktop' }: ClaudeAccountsPageProps) {
  const packages = usePlatformPackageStore((state) => state.packages);
  const initialized = usePlatformPackageStore((state) => state.initialized);
  const refreshPlatformPackages = usePlatformPackageStore((state) => state.refresh);
  const platformPackage = useMemo(
    () => getPlatformPackageFromPackages(packages, CLAUDE_PLATFORM_ID),
    [packages],
  );
  const runtimeParams = useMemo(
    () => ({ initialSubPlatform: subPlatform }),
    [subPlatform],
  );

  useEffect(() => {
    if (initialized) {
      return;
    }
    void refreshPlatformPackages().catch((error) => {
      console.error('Failed to refresh Claude platform package:', error);
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
    <div className="ghcp-accounts-page codex-accounts-page claude-accounts-page">
      <PlatformOverviewTabsHeader
        platform="claude_manager"
        active="overview"
        hideTabs
        remoteTabsSlotId={runtimeReady ? CLAUDE_REMOTE_TABS_SLOT_ID : undefined}
        rightSlot={<PlatformPackageToolbar platformId={CLAUDE_PLATFORM_ID} />}
      />

      {!runtimeReady ? (
        <PlatformPackageUnavailablePage platformId={CLAUDE_PLATFORM_ID} state={platformPackage} />
      ) : platformPackage ? (
        <PlatformRuntimePageHost
          platformId={CLAUDE_PLATFORM_ID}
          state={platformPackage}
          tabsSlotId={CLAUDE_REMOTE_TABS_SLOT_ID}
          runtimeParams={runtimeParams}
        />
      ) : (
        <PlatformPackageUnavailablePage platformId={CLAUDE_PLATFORM_ID} state={platformPackage} />
      )}
    </div>
  );
}
