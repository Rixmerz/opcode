import React, { useEffect, useState } from 'react';
import { useChunkingStore } from '@/stores/chunkingStore';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { ScrollArea } from '@/components/ui/scroll-area';
import {
  Database,
  Search,
  Filter,
  RefreshCw,
  Play,
  FileCode,
  GitBranch,
  TestTube,
  Settings,
  FileJson,
  Box,
  AlertCircle,
  Clock,
} from 'lucide-react';
import { ChunkGrid } from './ChunkGrid';
import { ChunkDetail } from './ChunkDetail';
import { BusinessRuleValidator } from './BusinessRuleValidator';
import { SnapshotTimeline } from './SnapshotTimeline';
import { ChunkErrorViewer } from './ChunkErrorViewer';
import { ChunkingProgress } from './ChunkingProgress';
import type { ChunkType } from '@/types/chunking';

interface ChunkExplorerProps {
  projectPath?: string;
  autoProcess?: boolean;
}

export const ChunkExplorer: React.FC<ChunkExplorerProps> = ({
  projectPath,
  autoProcess = false,
}) => {
  const {
    chunks,
    currentProjectPath,
    isProcessing,
    isLoadingChunks,
    error,
    stats,
    selectedChunk,
    searchQuery,
    selectedChunkTypes,
    processProject,
    refreshChunks,
    setSearchQuery,
    setSelectedChunkTypes,
    selectChunk,
    clearError,
  } = useChunkingStore();

  const [localSearchQuery, setLocalSearchQuery] = useState(searchQuery);

  // Auto-process on mount if enabled
  useEffect(() => {
    if (projectPath && autoProcess && !currentProjectPath) {
      processProject(projectPath);
    } else if (projectPath && currentProjectPath !== projectPath) {
      refreshChunks(projectPath);
    }
  }, [projectPath, autoProcess, currentProjectPath]);

  const handleProcess = () => {
    if (projectPath) {
      processProject(projectPath, {
        chunk_types: [
          'raw_source',
          'ast',
          'callgraph',
          'tests',
          'commit_history',
          'state_config',
          'project_metadata',
        ],
        include_dynamic_callgraph: false,
        max_commits: 100,
        ignore_patterns: ['node_modules/**', 'target/**', 'dist/**', 'build/**'],
      });
    }
  };

  const handleRefresh = () => {
    if (projectPath) {
      refreshChunks(projectPath);
    }
  };

  const handleSearch = () => {
    setSearchQuery(localSearchQuery);
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleSearch();
    }
  };

  const toggleChunkType = (type: ChunkType) => {
    const newTypes = selectedChunkTypes.includes(type)
      ? selectedChunkTypes.filter((t) => t !== type)
      : [...selectedChunkTypes, type];
    setSelectedChunkTypes(newTypes);
  };

  const chunkTypeIcons: Record<ChunkType, React.ReactNode> = {
    raw_source: <FileCode className="h-4 w-4" />,
    ast: <GitBranch className="h-4 w-4" />,
    callgraph: <Box className="h-4 w-4" />,
    tests: <TestTube className="h-4 w-4" />,
    commit_history: <Clock className="h-4 w-4" />,
    state_config: <Settings className="h-4 w-4" />,
    project_metadata: <FileJson className="h-4 w-4" />,
    business_rules: <AlertCircle className="h-4 w-4" />,
    snapshot: <Database className="h-4 w-4" />,
    error_log: <AlertCircle className="h-4 w-4" />,
  };

  return (
    <div className="flex flex-col h-full gap-4 p-4">
      {/* Header */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Database className="h-5 w-5" />
                Chunk Explorer
              </CardTitle>
              <CardDescription>
                Analyze and explore project chunks
                {currentProjectPath && (
                  <span className="ml-2 text-xs">({currentProjectPath})</span>
                )}
              </CardDescription>
            </div>
            <div className="flex gap-2">
              <Button onClick={handleRefresh} variant="outline" size="sm" disabled={isLoadingChunks}>
                <RefreshCw className={`h-4 w-4 mr-1 ${isLoadingChunks ? 'animate-spin' : ''}`} />
                Refresh
              </Button>
              <Button onClick={handleProcess} size="sm" disabled={isProcessing || !projectPath}>
                <Play className="h-4 w-4 mr-1" />
                Process Project
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          {/* Stats */}
          {stats && (
            <div className="flex gap-4 mb-4">
              <div className="text-sm">
                <span className="font-medium">Total Chunks:</span> {stats.totalChunks}
              </div>
              {stats.lastProcessed && (
                <div className="text-sm text-muted-foreground">
                  Last processed: {new Date(stats.lastProcessed).toLocaleString()}
                </div>
              )}
            </div>
          )}

          {/* Search and Filter */}
          <div className="flex gap-2 mb-4">
            <div className="flex-1 flex gap-2">
              <Input
                placeholder="Search chunks..."
                value={localSearchQuery}
                onChange={(e) => setLocalSearchQuery(e.target.value)}
                onKeyPress={handleKeyPress}
                className="flex-1"
              />
              <Button onClick={handleSearch} size="sm">
                <Search className="h-4 w-4" />
              </Button>
            </div>
          </div>

          {/* Chunk Type Filters */}
          <div className="flex flex-wrap gap-2">
            {(Object.keys(chunkTypeIcons) as ChunkType[]).map((type) => (
              <Badge
                key={type}
                variant={selectedChunkTypes.includes(type) ? 'default' : 'outline'}
                className="cursor-pointer"
                onClick={() => toggleChunkType(type)}
              >
                {chunkTypeIcons[type]}
                <span className="ml-1">{type.replace('_', ' ')}</span>
                {stats?.chunksByType[type] && (
                  <span className="ml-1">({stats.chunksByType[type]})</span>
                )}
              </Badge>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Processing Progress */}
      {isProcessing && <ChunkingProgress />}

      {/* Error Display */}
      {error && (
        <Card className="border-destructive">
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2 text-destructive">
                <AlertCircle className="h-4 w-4" />
                <span>{error}</span>
              </div>
              <Button onClick={clearError} variant="outline" size="sm">
                Dismiss
              </Button>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Main Content */}
      <div className="flex-1 min-h-0">
        <Tabs defaultValue="chunks" className="h-full flex flex-col">
          <TabsList>
            <TabsTrigger value="chunks">Chunks</TabsTrigger>
            <TabsTrigger value="business-rules">Business Rules</TabsTrigger>
            <TabsTrigger value="timeline">Timeline</TabsTrigger>
            <TabsTrigger value="errors">Errors</TabsTrigger>
          </TabsList>

          <TabsContent value="chunks" className="flex-1 min-h-0 flex gap-4">
            <div className="flex-1 min-w-0">
              <ScrollArea className="h-full">
                <ChunkGrid
                  chunks={chunks}
                  isLoading={isLoadingChunks}
                  onChunkSelect={selectChunk}
                  selectedChunkId={selectedChunk?.id}
                />
              </ScrollArea>
            </div>
            {selectedChunk && (
              <div className="w-1/2 min-w-0">
                <ChunkDetail chunk={selectedChunk} onClose={() => selectChunk(null)} />
              </div>
            )}
          </TabsContent>

          <TabsContent value="business-rules" className="flex-1 min-h-0">
            <BusinessRuleValidator projectPath={currentProjectPath || undefined} />
          </TabsContent>

          <TabsContent value="timeline" className="flex-1 min-h-0">
            <SnapshotTimeline projectPath={currentProjectPath || undefined} />
          </TabsContent>

          <TabsContent value="errors" className="flex-1 min-h-0">
            <ChunkErrorViewer projectPath={currentProjectPath || undefined} />
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
};
