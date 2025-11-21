import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Badge } from '@/components/ui/badge';
import { X, Copy, FileCode } from 'lucide-react';
import type { Chunk } from '@/types/chunking';

interface ChunkDetailProps {
  chunk: Chunk;
  onClose: () => void;
}

export const ChunkDetail: React.FC<ChunkDetailProps> = ({ chunk, onClose }) => {
  const handleCopy = () => {
    navigator.clipboard.writeText(chunk.content);
  };

  return (
    <Card className="h-full flex flex-col">
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-lg flex items-center gap-2">
            <FileCode className="h-5 w-5" />
            Chunk Details
          </CardTitle>
          <div className="flex gap-2">
            <Button onClick={handleCopy} variant="outline" size="sm">
              <Copy className="h-4 w-4" />
            </Button>
            <Button onClick={onClose} variant="ghost" size="sm">
              <X className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent className="flex-1 min-h-0 flex flex-col gap-4">
        <div className="space-y-2">
          <div className="flex items-center gap-2">
            <Badge>{chunk.chunk_type}</Badge>
            {chunk.file_path && <span className="text-sm text-muted-foreground">{chunk.file_path}</span>}
          </div>
          {chunk.entity_name && (
            <div className="text-sm">
              <span className="font-medium">Entity:</span> {chunk.entity_name}
            </div>
          )}
          <div className="text-xs text-muted-foreground">
            Created: {new Date(chunk.created_at).toLocaleString()}
          </div>
          <div className="text-xs text-muted-foreground">
            Updated: {new Date(chunk.updated_at).toLocaleString()}
          </div>
          <div className="text-xs text-muted-foreground">
            Hash: {chunk.content_hash.substring(0, 16)}...
          </div>
        </div>

        <div className="flex-1 min-h-0">
          <div className="text-sm font-medium mb-2">Content:</div>
          <ScrollArea className="h-full border rounded-md">
            <pre className="p-4 text-xs">
              <code>{chunk.content}</code>
            </pre>
          </ScrollArea>
        </div>

        {chunk.metadata && (
          <details className="text-sm">
            <summary className="cursor-pointer font-medium">Metadata</summary>
            <pre className="mt-2 p-2 bg-muted rounded text-xs overflow-auto">
              {JSON.stringify(JSON.parse(chunk.metadata), null, 2)}
            </pre>
          </details>
        )}
      </CardContent>
    </Card>
  );
};
