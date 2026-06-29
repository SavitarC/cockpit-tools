import type { ReactNode } from 'react';
import { PlatformOverviewTabsHeader } from './platform/PlatformOverviewTabsHeader';

interface ZedOverviewTabsHeaderProps {
  rightSlot?: ReactNode;
  hideTabs?: boolean;
  remoteTabsSlotId?: string;
}

export function ZedOverviewTabsHeader({
  rightSlot,
  hideTabs,
  remoteTabsSlotId,
}: ZedOverviewTabsHeaderProps) {
  return (
    <PlatformOverviewTabsHeader
      platform="zed"
      active="overview"
      tabs={['overview']}
      rightSlot={rightSlot}
      hideTabs={hideTabs}
      remoteTabsSlotId={remoteTabsSlotId}
    />
  );
}
