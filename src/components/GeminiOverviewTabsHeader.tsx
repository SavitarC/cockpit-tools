import { ReactNode } from 'react';
import { PlatformOverviewTabsHeader, PlatformOverviewTab } from './platform/PlatformOverviewTabsHeader';

export type GeminiTab = PlatformOverviewTab;

interface GeminiOverviewTabsHeaderProps {
  active: GeminiTab;
  onTabChange?: (tab: GeminiTab) => void;
  rightSlot?: ReactNode;
  hideTabs?: boolean;
  remoteTabsSlotId?: string;
}

export function GeminiOverviewTabsHeader({
  active,
  onTabChange,
  rightSlot,
  hideTabs,
  remoteTabsSlotId,
}: GeminiOverviewTabsHeaderProps) {
  return (
    <PlatformOverviewTabsHeader
      platform="gemini"
      active={active}
      onTabChange={onTabChange}
      rightSlot={rightSlot}
      hideTabs={hideTabs}
      remoteTabsSlotId={remoteTabsSlotId}
    />
  );
}
