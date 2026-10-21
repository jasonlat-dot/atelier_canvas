/** Core availability is separate from end-to-end product features. */
export type AgentCapabilities = {
  contractVersion: 1;
  mode: 'mock' | 'http';
  core: {toolArgumentValidation: boolean; contextBudgetPlanning: boolean};
  features: {chat: boolean; history: boolean; tools: boolean; approvals: boolean; compaction: boolean; memory: boolean};
};