import { useEffect, useRef } from 'react';
import { useChunkingStore } from '@/stores/chunkingStore';

interface UseAutoChunkingOptions {
  enabled?: boolean;
  projectPath?: string;
}

/**
 * Hook to automatically process chunks when a project is opened
 * @param options - Configuration options
 */
export const useAutoChunking = ({ enabled = true, projectPath }: UseAutoChunkingOptions) => {
  const { processProject, currentProjectPath, stats } = useChunkingStore();
  const hasProcessedRef = useRef<Set<string>>(new Set());

  useEffect(() => {
    if (!enabled || !projectPath) {
      return;
    }

    // Check if we've already processed this project in this session
    if (hasProcessedRef.current.has(projectPath)) {
      return;
    }

    // Check if project already has chunks (avoid re-processing on every open)
    if (currentProjectPath === projectPath && stats && stats.totalChunks > 0) {
      return;
    }

    // Mark as processed
    hasProcessedRef.current.add(projectPath);

    // Process the project with default options
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
      ignore_patterns: [
        'node_modules/**',
        'target/**',
        'dist/**',
        'build/**',
        '.git/**',
        'coverage/**',
        '.next/**',
        'out/**',
      ],
    });
  }, [enabled, projectPath, currentProjectPath, stats, processProject]);
};
