import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import type { Chunk } from '@/types/chunking';
import { FileCode, GitBranch, Box, TestTube, Clock, Settings, FileJson, AlertCircle } from 'lucide-react';

interface ChunkGridProps {
  chunks: Chunk[];
  isLoading?: boolean;
  onChunkSelect: (chunk: Chunk) => void;
  selectedChunkId?: number;
}

export const ChunkGrid: React.FC<ChunkGridProps> = ({
  chunks,
  isLoading,
  onChunkSelect,
  selectedChunkId,
}) => {
  const getChunkIcon = (type: string) => {
    const icons: Record<string, React.ReactNode> = {
      raw_source: <FileCode className="h-4 w-4" />,
      ast: <GitBranch className="h-4 w-4" />,
      callgraph: <Box className="h-4 w-4" />,
      tests: <TestTube className="h-4 w-4" />,
      commit_history: <Clock className="h-4 w-4" />,
      state_config: <Settings className="h-4 w-4" />,
      project_metadata: <FileJson className="h-4 w-4" />,
      business_rules: <AlertCircle className="h-4 w-4" />,
    };
    return icons[type] || <FileCode className="h-4 w-4" />;
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-muted-foreground">Loading chunks...</div>
      </div>
    );
  }

  if (chunks.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-64 text-muted-foreground">
        <Box className="h-12 w-12 mb-4 opacity-50" />
        <p>No chunks found</p>
        <p className="text-sm">Process a project to generate chunks</p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {chunks.map((chunk) => (
        <Card
          key={chunk.id}
          className={`cursor-pointer transition-all hover:shadow-md ${
            selectedChunkId === chunk.id ? 'ring-2 ring-primary' : ''
          }`}
          onClick={() => onChunkSelect(chunk)}
        >
          <CardHeader className="pb-3">
            <div className="flex items-start justify-between gap-2">
              <div className="flex items-center gap-2 flex-1 min-w-0">
                {getChunkIcon(chunk.chunk_type)}
                <Badge variant="secondary" className="text-xs">
                  {chunk.chunk_type}
                </Badge>
              </div>
            </div>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {chunk.file_path && (
                <div className="text-sm truncate" title={chunk.file_path}>
                  <span className="text-muted-foreground">File:</span> {chunk.file_path}
                </div>
              )}
              {chunk.entity_name && (
                <div className="text-sm truncate" title={chunk.entity_name}>
                  <span className="text-muted-foreground">Entity:</span> {chunk.entity_name}
                </div>
              )}
              <div className="text-xs text-muted-foreground">
                {chunk.content.length} characters
              </div>
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
};
