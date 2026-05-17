import { callCommand } from "./client";
import type {
  PromptTemplate,
  PromptTemplateDraft,
  PromptTemplatePreview,
  PromptTemplateVariableValue,
} from "@/types/ai";

export function listPromptTemplates(): Promise<PromptTemplate[]> {
  return callCommand<PromptTemplate[]>("list_prompt_templates").then(
    (items) => items ?? [],
  );
}

export function copyPromptTemplate(
  templateId: string,
): Promise<PromptTemplate> {
  return callCommand<PromptTemplate>("copy_prompt_template", { templateId });
}

export function savePromptTemplate(
  draft: PromptTemplateDraft,
): Promise<PromptTemplate> {
  return callCommand<PromptTemplate>("save_prompt_template", { draft });
}

export function resetBuiltinPromptTemplates(): Promise<PromptTemplate[]> {
  return callCommand<PromptTemplate[]>("reset_builtin_prompt_templates").then(
    (items) => items ?? [],
  );
}

export function previewPromptTemplate(
  template: string,
  values: PromptTemplateVariableValue[],
): Promise<PromptTemplatePreview> {
  return callCommand<PromptTemplatePreview>("preview_prompt_template", {
    request: { template, values },
  });
}
