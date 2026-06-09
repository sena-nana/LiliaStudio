import { executeCommand, listResult } from "./client";
import { commands } from "./commands";
import type {
  PromptTemplate,
  PromptTemplateDraft,
  PromptTemplatePreview,
  PromptTemplateVariableValue,
} from "@/types/ai";

export function listPromptTemplates(): Promise<PromptTemplate[]> {
  return executeCommand(commands.prompt.listTemplates).then(listResult);
}

export function copyPromptTemplate(
  templateId: string,
): Promise<PromptTemplate> {
  return executeCommand(commands.prompt.copyTemplate, { templateId });
}

export function savePromptTemplate(
  draft: PromptTemplateDraft,
): Promise<PromptTemplate> {
  return executeCommand(commands.prompt.saveTemplate, { draft });
}

export function resetBuiltinPromptTemplates(): Promise<PromptTemplate[]> {
  return executeCommand(commands.prompt.resetBuiltinTemplates).then(listResult);
}

export function previewPromptTemplate(
  template: string,
  values: PromptTemplateVariableValue[],
): Promise<PromptTemplatePreview> {
  return executeCommand(commands.prompt.previewTemplate, {
    request: { template, values },
  });
}
