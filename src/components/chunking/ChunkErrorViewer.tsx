import React, { useEffect } from 'react';
import { useChunkingStore } from '@/stores/chunkingStore';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import { AlertCircle, CheckCircle, FileCode, Hash } from 'lucide-react';
import type { ErrorLog } from '@/types/chunking';

interface ChunkErrorViewerProps {
  projectPath?: string;
}

export const ChunkErrorViewer: React.FC<ChunkErrorViewerProps> = ({ projectPath }) => {
  const { errors, isLoadingErrors, fetchErrors, resolveError } = useChunkingStore();

  useEffect(() => {
    if (projectPath) {
      fetchErrors(projectPath);
    }
  }, [projectPath]);

  const handleResolve = async (errorId: number) => {
    await resolveError(errorId);
  };

  const renderError = (error: ErrorLog) => (
    <Card key={error.id} className="mb-4">
      <CardHeader>
        <div className="flex items-start justify-between">
          <div className="flex items-start gap-2 flex-1">
            <AlertCircle className="h-5 w-5 text-destructive mt-0.5" />
            <div className="flex-1">
              <div className="flex items-center gap-2 mb-1">
                <Badge variant="destructive">{error.error_type}</Badge>
                {error.occurrence_count > 1 && (
                  <Badge variant="secondary">
                    <Hash className="h-3 w-3 mr-1" />
                    {error.occurrence_count}x
                  </Badge>
                )}
              </div>
              <CardTitle className="text-base">{error.message}</CardTitle>
              {error.file_path && (
                <CardDescription className="mt-1 flex items-center gap-1">
                  <FileCode className="h-3 w-3" />
                  {error.file_path}
                  {error.entity_name && ` :: ${error.entity_name}`}
                </CardDescription>
              )}
            </div>
          </div>
          <Button onClick={() => handleResolve(error.id!)} size="sm" variant="outline">
            <CheckCircle className="h-4 w-4 mr-1" />
            Resolve
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-2">
          <div className="text-xs text-muted-foreground">
            First seen: {new Date(error.first_seen).toLocaleString()}
          </div>
          <div className="text-xs text-muted-foreground">
            Last seen: {new Date(error.last_seen).toLocaleString()}
          </div>
          {error.stacktrace && (
            <details className="mt-2">
              <summary className="cursor-pointer text-sm font-medium">Stacktrace</summary>
              <pre className="mt-2 p-2 bg-muted rounded text-xs overflow-auto max-h-64">
                {error.stacktrace}
              </pre>
            </details>
          )}
        </div>
      </CardContent>
    </Card>
  );

  if (isLoadingErrors) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-muted-foreground">Loading errors...</div>
      </div>
    );
  }

  return (
    <ScrollArea className="h-full">
      {errors.length > 0 ? (
        <div className="space-y-4">
          {errors.map(renderError)}
        </div>
      ) : (
        <Card>
          <CardContent className="pt-6">
            <div className="flex flex-col items-center justify-center h-64 text-muted-foreground">
              <CheckCircle className="h-12 w-12 mb-4 opacity-50 text-green-500" />
              <p>No active errors</p>
              <p className="text-sm">All errors have been resolved</p>
            </div>
          </CardContent>
        </Card>
      )}
    </ScrollArea>
  );
};
