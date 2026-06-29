import type { ReactNode } from 'react';
import { PlatformOverviewTabsHeader, PlatformOverviewTab } from './platform/PlatformOverviewTabsHeader';

export type KiroTab = PlatformOverviewTab;

interface KiroOverviewTabsHeaderProps {
  active?: KiroTab;
  onTabChange?: (tab: KiroTab) => void;
  rightSlot?: ReactNode;
  hideTabs?: boolean;
  remoteTabsSlotId?: string;
}

export function KiroOverviewTabsHeader({
  active = 'overview',
  onTabChange,
  rightSlot,
  hideTabs,
  remoteTabsSlotId,
}: KiroOverviewTabsHeaderProps) {
  return (
    <PlatformOverviewTabsHeader
      platform="kiro"
      active={active}
      onTabChange={onTabChange}
      tabs={['overview', 'instances']}
      rightSlot={rightSlot}
      hideTabs={hideTabs}
      remoteTabsSlotId={remoteTabsSlotId}
    />
  );
}
