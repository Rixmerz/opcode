import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import type { StateCreator } from 'zustand';
import { api } from '@/lib/api';
import type {
  Chunk,
  ChunkType,
  ChunkQuery,
  ChunkingOptions,
  ChunkingResult,
  BusinessRule,
  Snapshot,
  SnapshotType,
  ErrorLog,
  ChunkFilterOptions,
  ChunkStats,
} from '@/types/chunking';

interface ChunkingState {
  // Data
  chunks: Chunk[];
  currentProjectPath: string | null;
  selectedChunk: Chunk | null;
  businessRules: BusinessRule[];
  pendingBusinessRules: BusinessRule[];
  snapshots: {
    master: Snapshot[];
    agent: Snapshot[];
  };
  errors: ErrorLog[];

  // Processing state
  isProcessing: boolean;
  processingResult: ChunkingResult | null;
  processingProgress: number;

  // UI state
  searchQuery: string;
  selectedChunkTypes: ChunkType[];
  expandedChunkIds: Set<string>;
  filterOptions: ChunkFilterOptions;

  // Loading states
  isLoadingChunks: boolean;
  isLoadingBusinessRules: boolean;
  isLoadingSnapshots: boolean;
  isLoadingErrors: boolean;

  // Error state
  error: string | null;

  // Stats
  stats: ChunkStats | null;
  lastProcessedAt: string | null;

  // Actions - Processing
  processProject: (projectPath: string, options?: ChunkingOptions) => Promise<void>;
  refreshChunks: (projectPath: string) => Promise<void>;

  // Actions - Search & Filter
  searchChunks: (query: ChunkQuery) => Promise<void>;
  setSearchQuery: (query: string) => void;
  setSelectedChunkTypes: (types: ChunkType[]) => void;
  setFilterOptions: (options: Partial<ChunkFilterOptions>) => void;
  clearFilters: () => void;

  // Actions - Chunk Selection
  selectChunk: (chunk: Chunk | null) => void;
  toggleChunkExpanded: (chunkId: string) => void;

  // Actions - Business Rules
  fetchPendingBusinessRules: (projectPath: string) => Promise<void>;
  validateBusinessRule: (ruleId: number, description: string, correction?: string) => Promise<void>;
  proposeBusinessRule: (
    projectPath: string,
    entityName: string,
    filePath: string,
    interpretation: string
  ) => Promise<void>;

  // Actions - Snapshots
  fetchSnapshots: (projectPath: string, type?: SnapshotType) => Promise<void>;
  createMasterSnapshot: (
    projectPath: string,
    userMessage: string,
    changedFiles: string[]
  ) => Promise<void>;
  createAgentSnapshot: (
    projectPath: string,
    message: string,
    changedFiles: string[]
  ) => Promise<void>;

  // Actions - Errors
  fetchErrors: (projectPath: string) => Promise<void>;
  logError: (
    projectPath: string,
    errorType: string,
    message: string,
    filePath?: string,
    stacktrace?: string
  ) => Promise<void>;
  resolveError: (errorId: number) => Promise<void>;

  // Actions - Utilities
  clearError: () => void;
  reset: () => void;
}

const defaultFilterOptions: ChunkFilterOptions = {
  chunkTypes: [],
  searchQuery: '',
};

const chunkingStore: StateCreator<
  ChunkingState,
  [],
  [['zustand/subscribeWithSelector', never]],
  ChunkingState
> = (set, get) => ({
  // Initial state
  chunks: [],
  currentProjectPath: null,
  selectedChunk: null,
  businessRules: [],
  pendingBusinessRules: [],
  snapshots: {
    master: [],
    agent: [],
  },
  errors: [],

  isProcessing: false,
  processingResult: null,
  processingProgress: 0,

  searchQuery: '',
  selectedChunkTypes: [],
  expandedChunkIds: new Set(),
  filterOptions: defaultFilterOptions,

  isLoadingChunks: false,
  isLoadingBusinessRules: false,
  isLoadingSnapshots: false,
  isLoadingErrors: false,

  error: null,
  stats: null,
  lastProcessedAt: null,

  // Process project and generate chunks
  processProject: async (projectPath: string, options?: ChunkingOptions) => {
    set({ isProcessing: true, error: null, processingProgress: 0 });
    try {
      const result = await api.processProjectChunks(projectPath, options);
      set({
        processingResult: result,
        currentProjectPath: projectPath,
        lastProcessedAt: result.completed_at,
        isProcessing: false,
        processingProgress: 100,
      });

      // Refresh chunks after processing
      await get().refreshChunks(projectPath);
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to process project',
        isProcessing: false,
      });
    }
  },

  // Refresh chunks from database
  refreshChunks: async (projectPath: string) => {
    set({ isLoadingChunks: true, error: null });
    try {
      const chunks = await api.searchChunks({ project_path: projectPath });

      // Calculate stats
      const chunksByType: Record<ChunkType, number> = {} as any;
      chunks.forEach((chunk) => {
        chunksByType[chunk.chunk_type] = (chunksByType[chunk.chunk_type] || 0) + 1;
      });

      const stats: ChunkStats = {
        totalChunks: chunks.length,
        chunksByType,
        lastProcessed: get().lastProcessedAt || undefined,
      };

      set({
        chunks,
        currentProjectPath: projectPath,
        stats,
        isLoadingChunks: false,
      });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to refresh chunks',
        isLoadingChunks: false,
      });
    }
  },

  // Search chunks with filters
  searchChunks: async (query: ChunkQuery) => {
    set({ isLoadingChunks: true, error: null });
    try {
      const chunks = await api.searchChunks(query);
      set({ chunks, isLoadingChunks: false });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to search chunks',
        isLoadingChunks: false,
      });
    }
  },

  // Set search query
  setSearchQuery: (query: string) => {
    set({ searchQuery: query });
    const { currentProjectPath, selectedChunkTypes } = get();
    if (currentProjectPath) {
      get().searchChunks({
        project_path: currentProjectPath,
        chunk_types: selectedChunkTypes.length > 0 ? selectedChunkTypes : undefined,
      });
    }
  },

  // Set selected chunk types filter
  setSelectedChunkTypes: (types: ChunkType[]) => {
    set({ selectedChunkTypes: types });
    const { currentProjectPath } = get();
    if (currentProjectPath) {
      get().searchChunks({
        project_path: currentProjectPath,
        chunk_types: types.length > 0 ? types : undefined,
      });
    }
  },

  // Set filter options
  setFilterOptions: (options: Partial<ChunkFilterOptions>) => {
    set((state) => ({
      filterOptions: { ...state.filterOptions, ...options },
    }));
  },

  // Clear all filters
  clearFilters: () => {
    set({
      searchQuery: '',
      selectedChunkTypes: [],
      filterOptions: defaultFilterOptions,
    });
    const { currentProjectPath } = get();
    if (currentProjectPath) {
      get().refreshChunks(currentProjectPath);
    }
  },

  // Select a chunk
  selectChunk: (chunk: Chunk | null) => {
    set({ selectedChunk: chunk });
  },

  // Toggle chunk expansion
  toggleChunkExpanded: (chunkId: string) => {
    set((state) => {
      const newSet = new Set(state.expandedChunkIds);
      if (newSet.has(chunkId)) {
        newSet.delete(chunkId);
      } else {
        newSet.add(chunkId);
      }
      return { expandedChunkIds: newSet };
    });
  },

  // Fetch pending business rules
  fetchPendingBusinessRules: async (projectPath: string) => {
    set({ isLoadingBusinessRules: true, error: null });
    try {
      const rules = await api.getPendingBusinessRules(projectPath);
      set({
        pendingBusinessRules: rules,
        isLoadingBusinessRules: false,
      });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to fetch business rules',
        isLoadingBusinessRules: false,
      });
    }
  },

  // Validate a business rule
  validateBusinessRule: async (ruleId: number, description: string, correction?: string) => {
    try {
      await api.validateBusinessRule(ruleId, description, correction);
      // Refresh pending rules
      const { currentProjectPath } = get();
      if (currentProjectPath) {
        await get().fetchPendingBusinessRules(currentProjectPath);
      }
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to validate business rule',
      });
    }
  },

  // Propose a new business rule
  proposeBusinessRule: async (
    projectPath: string,
    entityName: string,
    filePath: string,
    interpretation: string
  ) => {
    try {
      await api.proposeBusinessRule(projectPath, entityName, filePath, interpretation);
      // Refresh pending rules
      await get().fetchPendingBusinessRules(projectPath);
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to propose business rule',
      });
    }
  },

  // Fetch snapshots
  fetchSnapshots: async (projectPath: string, type?: SnapshotType) => {
    set({ isLoadingSnapshots: true, error: null });
    try {
      const snapshots = await api.getProjectSnapshots(projectPath, type);

      if (type) {
        set((state) => ({
          snapshots: {
            ...state.snapshots,
            [type]: snapshots,
          },
          isLoadingSnapshots: false,
        }));
      } else {
        // Fetch both types
        const masterSnapshots = await api.getProjectSnapshots(projectPath, 'master');
        const agentSnapshots = await api.getProjectSnapshots(projectPath, 'agent');
        set({
          snapshots: {
            master: masterSnapshots,
            agent: agentSnapshots,
          },
          isLoadingSnapshots: false,
        });
      }
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to fetch snapshots',
        isLoadingSnapshots: false,
      });
    }
  },

  // Create master snapshot
  createMasterSnapshot: async (
    projectPath: string,
    userMessage: string,
    changedFiles: string[]
  ) => {
    try {
      await api.createMasterSnapshot(projectPath, userMessage, changedFiles);
      // Refresh snapshots
      await get().fetchSnapshots(projectPath, 'master');
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to create master snapshot',
      });
    }
  },

  // Create agent snapshot
  createAgentSnapshot: async (
    projectPath: string,
    message: string,
    changedFiles: string[]
  ) => {
    try {
      await api.createAgentSnapshot(projectPath, message, changedFiles);
      // Refresh snapshots
      await get().fetchSnapshots(projectPath, 'agent');
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to create agent snapshot',
      });
    }
  },

  // Fetch errors
  fetchErrors: async (projectPath: string) => {
    set({ isLoadingErrors: true, error: null });
    try {
      const errors = await api.getProjectErrors(projectPath);
      set({ errors, isLoadingErrors: false });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to fetch errors',
        isLoadingErrors: false,
      });
    }
  },

  // Log an error
  logError: async (
    projectPath: string,
    errorType: string,
    message: string,
    filePath?: string,
    stacktrace?: string
  ) => {
    try {
      await api.logError(projectPath, errorType, message, filePath, stacktrace);
      // Refresh errors
      await get().fetchErrors(projectPath);
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to log error',
      });
    }
  },

  // Resolve an error
  resolveError: async (errorId: number) => {
    try {
      await api.resolveError(errorId);
      // Refresh errors
      const { currentProjectPath } = get();
      if (currentProjectPath) {
        await get().fetchErrors(currentProjectPath);
      }
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to resolve error',
      });
    }
  },

  // Clear error
  clearError: () => {
    set({ error: null });
  },

  // Reset state
  reset: () => {
    set({
      chunks: [],
      currentProjectPath: null,
      selectedChunk: null,
      businessRules: [],
      pendingBusinessRules: [],
      snapshots: { master: [], agent: [] },
      errors: [],
      isProcessing: false,
      processingResult: null,
      processingProgress: 0,
      searchQuery: '',
      selectedChunkTypes: [],
      expandedChunkIds: new Set(),
      filterOptions: defaultFilterOptions,
      error: null,
      stats: null,
      lastProcessedAt: null,
    });
  },
});

export const useChunkingStore = create<ChunkingState>()(
  subscribeWithSelector(chunkingStore)
);
