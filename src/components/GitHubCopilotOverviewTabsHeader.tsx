import { ReactNode } from 'react';
import { PlatformOverviewTabsHeader, PlatformOverviewTab } from './platform/PlatformOverviewTabsHeader';

export type GitHubCopilotTab = PlatformOverviewTab;

interface GitHubCopilotOverviewTabsHeaderProps {
  active: GitHubCopilotTab;
  onTabChange?: (tab: GitHubCopilotTab) => void;
  rightSlot?: ReactNode;
  hideTabs?: boolean;
  remoteTabsSlotId?: string;
}

export function GitHubCopilotOverviewTabsHeader({
  active,
  onTabChange,
  rightSlot,
  hideTabs,
  remoteTabsSlotId,
}: GitHubCopilotOverviewTabsHeaderProps) {
  return (
    <PlatformOverviewTabsHeader
      platform="github-copilot"
      active={active}
      onTabChange={onTabChange}
      rightSlot={rightSlot}
      hideTabs={hideTabs}
      remoteTabsSlotId={remoteTabsSlotId}
    />
  );
}
