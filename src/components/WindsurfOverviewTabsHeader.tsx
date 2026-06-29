import { ReactNode } from 'react';
import { PlatformOverviewTabsHeader, PlatformOverviewTab } from './platform/PlatformOverviewTabsHeader';

export type WindsurfTab = PlatformOverviewTab;

interface WindsurfOverviewTabsHeaderProps {
  active: WindsurfTab;
  onTabChange?: (tab: WindsurfTab) => void;
  rightSlot?: ReactNode;
  hideTabs?: boolean;
  remoteTabsSlotId?: string;
}

export function WindsurfOverviewTabsHeader({
  active,
  onTabChange,
  rightSlot,
  hideTabs,
  remoteTabsSlotId,
}: WindsurfOverviewTabsHeaderProps) {
  return (
    <PlatformOverviewTabsHeader
      platform="windsurf"
      active={active}
      onTabChange={onTabChange}
      rightSlot={rightSlot}
      hideTabs={hideTabs}
      remoteTabsSlotId={remoteTabsSlotId}
    />
  );
}
