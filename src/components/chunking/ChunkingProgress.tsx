import React from 'react';
import { Card, CardContent } from '@/components/ui/card';
import { useChunkingStore } from '@/stores/chunkingStore';
import { Loader2 } from 'lucide-react';

export const ChunkingProgress: React.FC = () => {
  const { processingProgress, processingResult } = useChunkingStore();

  return (
    <Card>
      <CardContent className="pt-6">
        <div className="flex items-center gap-4">
          <Loader2 className="h-6 w-6 animate-spin text-primary" />
          <div className="flex-1">
            <div className="text-sm font-medium">Processing project chunks...</div>
            {processingResult && (
              <div className="text-xs text-muted-foreground mt-1">
                {processingResult.chunks_created} chunks created
              </div>
            )}
          </div>
          <div className="text-sm font-medium">{processingProgress}%</div>
        </div>
        <div className="mt-2 h-2 bg-secondary rounded-full overflow-hidden">
          <div
            className="h-full bg-primary transition-all duration-300"
            style={{ width: `${processingProgress}%` }}
          />
        </div>
      </CardContent>
    </Card>
  );
};
