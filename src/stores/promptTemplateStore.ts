import { defineStore } from "pinia";
import {
  copyPromptTemplate,
  listPromptTemplates,
  previewPromptTemplate,
  resetBuiltinPromptTemplates,
  savePromptTemplate,
} from "@/api/promptTemplates";
import type {
  PromptTemplate,
  PromptTemplateDraft,
  PromptTemplatePreview,
  PromptTemplateVariableValue,
} from "@/types/ai";

interface PromptTemplateState {
  templates: PromptTemplate[];
  selectedTemplate: PromptTemplate | null;
  previewResult: PromptTemplatePreview | null;
  loading: boolean;
}

export const usePromptTemplateStore = defineStore("promptTemplates", {
  state: (): PromptTemplateState => ({
    templates: [],
    selectedTemplate: null,
    previewResult: null,
    loading: false,
  }),
  actions: {
    async loadTemplates() {
      this.templates = await listPromptTemplates();
      if (
        !this.selectedTemplate ||
        !this.templates.some((item) => item.id === this.selectedTemplate?.id)
      ) {
        this.selectedTemplate = this.templates[0] ?? null;
      }
      return this.templates;
    },
    selectTemplate(templateId: string) {
      this.selectedTemplate =
        this.templates.find((template) => template.id === templateId) ?? null;
      this.previewResult = null;
    },
    async copyTemplate(templateId: string) {
      const template = await copyPromptTemplate(templateId);
      this.templates = [template, ...this.templates];
      this.selectedTemplate = template;
      return template;
    },
    async saveTemplate(draft: PromptTemplateDraft) {
      const template = await savePromptTemplate(draft);
      this.replaceTemplate(template);
      this.selectedTemplate = template;
      return template;
    },
    async resetBuiltins() {
      const builtins = await resetBuiltinPromptTemplates();
      const customTemplates = this.templates.filter(
        (template) => !template.builtIn,
      );
      this.templates = [...builtins, ...customTemplates];
      if (this.selectedTemplate?.builtIn) {
        this.selectedTemplate = builtins[0] ?? null;
      }
      return builtins;
    },
    async preview(template: string, values: PromptTemplateVariableValue[]) {
      this.previewResult = await previewPromptTemplate(template, values);
      return this.previewResult;
    },
    replaceTemplate(template: PromptTemplate) {
      const index = this.templates.findIndex((item) => item.id === template.id);
      if (index >= 0) {
        this.templates.splice(index, 1, template);
      } else {
        this.templates = [template, ...this.templates];
      }
    },
  },
});
