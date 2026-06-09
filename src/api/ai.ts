import { executeCommand, listResult } from "./client";
import { commands } from "./commands";
import type {
  AiProviderConfig,
  AiProviderSettingsDraft,
  AiProviderSettingsView,
  CliProviderTestResult,
  ContextPack,
  DocumentChunkRecord,
  OpenAiProviderTestResult,
  TextChunk,
} from "@/types/ai";

export function defaultAiProviders(): Promise<AiProviderConfig[]> {
  return executeCommand(commands.ai.defaultProviders).then(listResult);
}

export function loadAiProviderSettings(): Promise<AiProviderSettingsView[]> {
  return executeCommand(commands.ai.loadProviderSettings).then(listResult);
}

export function saveAiProviderSettings(
  drafts: AiProviderSettingsDraft[],
): Promise<AiProviderSettingsView[]> {
  return executeCommand(commands.ai.saveProviderSettings, {
    drafts,
  }).then(listResult);
}

export function testOpenAiProvider(): Promise<OpenAiProviderTestResult> {
  return executeCommand(commands.ai.testOpenAiProvider);
}

export function testCodexCliProvider(): Promise<CliProviderTestResult> {
  return executeCommand(commands.ai.testCodexCliProvider);
}

export function testClaudeCliProvider(): Promise<CliProviderTestResult> {
  return executeCommand(commands.ai.testClaudeCliProvider);
}

export function previewChunks(
  text: string,
  maxChars: number,
): Promise<TextChunk[]> {
  return executeCommand(commands.vector.previewChunks, { text, maxChars }).then(listResult);
}

export function indexChunks(
  projectId: string,
  maxChars: number,
): Promise<DocumentChunkRecord[]> {
  return executeCommand(commands.rag.indexChunks, {
    projectId,
    maxChars,
  }).then(listResult);
}

export function previewContextPack(
  projectId: string,
  query: string,
  queryVector: number[],
): Promise<ContextPack> {
  return executeCommand(commands.rag.previewContextPack, {
    projectId,
    query,
    queryVector,
  });
}
