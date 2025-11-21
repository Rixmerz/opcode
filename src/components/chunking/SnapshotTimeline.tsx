import React, { useEffect } from 'react';
import { useChunkingStore } from '@/stores/chunkingStore';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { User, Bot, Clock, FileText } from 'lucide-react';
import type { Snapshot } from '@/types/chunking';

interface SnapshotTimelineProps {
  projectPath?: string;
}

export const SnapshotTimeline: React.FC<SnapshotTimelineProps> = ({ projectPath }) => {
  const { snapshots, isLoadingSnapshots, fetchSnapshots } = useChunkingStore();

  useEffect(() => {
    if (projectPath) {
      fetchSnapshots(projectPath);
    }
  }, [projectPath]);

  const renderSnapshot = (snapshot: Snapshot) => {
    const changedFiles = JSON.parse(snapshot.changed_files) as string[];
    const isMaster = snapshot.snapshot_type === 'master';

    return (
      <Card key={snapshot.id} className="mb-4">
        <CardHeader>
          <div className="flex items-start justify-between">
            <div className="flex items-start gap-2 flex-1">
              {isMaster ? (
                <User className="h-5 w-5 text-primary mt-0.5" />
              ) : (
                <Bot className="h-5 w-5 text-muted-foreground mt-0.5" />
              )}
              <div className="flex-1">
                <CardTitle className="text-base">{snapshot.message}</CardTitle>
                {snapshot.user_message && (
                  <CardDescription className="mt-1">{snapshot.user_message}</CardDescription>
                )}
              </div>
            </div>
            <Badge variant={isMaster ? 'default' : 'secondary'}>
              {snapshot.snapshot_type}
            </Badge>
          </div>
        </CardHeader>
        <CardContent>
          <div className="space-y-2">
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              <Clock className="h-3 w-3" />
              {new Date(snapshot.created_at).toLocaleString()}
            </div>
            {changedFiles.length > 0 && (
              <div>
                <div className="text-sm font-medium mb-1">
                  <FileText className="h-3 w-3 inline mr-1" />
                  Changed Files ({changedFiles.length}):
                </div>
                <div className="space-y-1">
                  {changedFiles.slice(0, 5).map((file, idx) => (
                    <div key={idx} className="text-xs text-muted-foreground truncate">
                      • {file}
                    </div>
                  ))}
                  {changedFiles.length > 5 && (
                    <div className="text-xs text-muted-foreground">
                      ... and {changedFiles.length - 5} more
                    </div>
                  )}
                </div>
              </div>
            )}
            {snapshot.diff_summary && (
              <div className="text-sm bg-muted p-2 rounded-sm mt-2">
                {snapshot.diff_summary}
              </div>
            )}
          </div>
        </CardContent>
      </Card>
    );
  };

  if (isLoadingSnapshots) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-muted-foreground">Loading snapshots...</div>
      </div>
    );
  }

  return (
    <Tabs defaultValue="master" className="h-full">
      <TabsList>
        <TabsTrigger value="master" className="flex items-center gap-2">
          <User className="h-4 w-4" />
          User Intent ({snapshots.master.length})
        </TabsTrigger>
        <TabsTrigger value="agent" className="flex items-center gap-2">
          <Bot className="h-4 w-4" />
          Agent Execution ({snapshots.agent.length})
        </TabsTrigger>
      </TabsList>

      <TabsContent value="master" className="h-[calc(100%-50px)]">
        <ScrollArea className="h-full">
          {snapshots.master.length > 0 ? (
            <div className="space-y-4">
              {snapshots.master.map(renderSnapshot)}
            </div>
          ) : (
            <Card>
              <CardContent className="pt-6">
                <div className="flex flex-col items-center justify-center h-64 text-muted-foreground">
                  <User className="h-12 w-12 mb-4 opacity-50" />
                  <p>No user snapshots yet</p>
                </div>
              </CardContent>
            </Card>
          )}
        </ScrollArea>
      </TabsContent>

      <TabsContent value="agent" className="h-[calc(100%-50px)]">
        <ScrollArea className="h-full">
          {snapshots.agent.length > 0 ? (
            <div className="space-y-4">
              {snapshots.agent.map(renderSnapshot)}
            </div>
          ) : (
            <Card>
              <CardContent className="pt-6">
                <div className="flex flex-col items-center justify-center h-64 text-muted-foreground">
                  <Bot className="h-12 w-12 mb-4 opacity-50" />
                  <p>No agent snapshots yet</p>
                </div>
              </CardContent>
            </Card>
          )}
        </ScrollArea>
      </TabsContent>
    </Tabs>
  );
};
