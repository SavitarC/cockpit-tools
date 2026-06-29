import { ReactNode } from 'react';
import { PlatformOverviewTabsHeader, PlatformOverviewTab } from './platform/PlatformOverviewTabsHeader';

export type CursorTab = PlatformOverviewTab;

interface CursorOverviewTabsHeaderProps {
  active: CursorTab;
  onTabChange?: (tab: CursorTab) => void;
  rightSlot?: ReactNode;
  hideTabs?: boolean;
  remoteTabsSlotId?: string;
}

export function CursorOverviewTabsHeader({
  active,
  onTabChange,
  rightSlot,
  hideTabs,
  remoteTabsSlotId,
}: CursorOverviewTabsHeaderProps) {
  return (
    <PlatformOverviewTabsHeader
      platform="cursor"
      active={active}
      onTabChange={onTabChange}
      rightSlot={rightSlot}
      hideTabs={hideTabs}
      remoteTabsSlotId={remoteTabsSlotId}
    />
  );
}
