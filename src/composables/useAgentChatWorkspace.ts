import { computed, onMounted, ref, watch } from "vue";
import { askAgent } from "@/api/ai";
import { entityTypeLabel } from "@/domain/displayLabels";
import { useJobStore } from "@/stores/jobStore";
import { useLibraryStore } from "@/stores/libraryStore";
import { useProjectStore } from "@/stores/projectStore";
import { useSearchStore } from "@/stores/searchStore";
import type { AgentAskReference, AgentAskResponse } from "@/types/ai";

interface SearchReferenceInput {
  entityType: string;
  entityId: string;
  title: string;
  snippet: string;
  source: string;
  score: number;
}

export function useAgentChatWorkspace() {
  const projectStore = useProjectStore();
  const searchStore = useSearchStore();
  const libraryStore = useLibraryStore();
  const jobStore = useJobStore();

  const selectedProjectId = ref("");
  const question = ref("");
  const asking = ref(false);
  const savingReport = ref(false);
  const savingTask = ref(false);
  const errorMessage = ref("");
  const statusMessage = ref("");
  const contextReferences = ref<AgentAskReference[]>([]);
  const lastResponse = ref<AgentAskResponse | null>(null);

  const answer = computed(() => lastResponse.value?.answer ?? "");
  const references = computed(() => lastResponse.value?.references ?? contextReferences.value);
  const canSubmit = computed(() => Boolean(selectedProjectId.value && question.value.trim()) && !asking.value);
  const canSave = computed(() => Boolean(selectedProjectId.value && answer.value && lastResponse.value));
  const semanticNotice = computed(() => {
    if (searchStore.semanticStatus === "degraded" || searchStore.semanticStatus === "failed") {
      return searchStore.semanticMessage;
    }
    return "";
  });

  onMounted(async () => {
    if (projectStore.projects.length === 0) {
      await projectStore.loadProjects();
    }
    selectedProjectId.value = projectStore.activeProjectId ?? projectStore.projects[0]?.id ?? "";
  });

  watch(selectedProjectId, (projectId) => {
    if (projectId) {
      projectStore.selectProject(projectId);
    }
  });

  async function submitQuestion() {
    const projectId = selectedProjectId.value;
    const text = question.value.trim();
    if (!projectId || !text) return;

    asking.value = true;
    errorMessage.value = "";
    statusMessage.value = "";
    contextReferences.value = [];
    lastResponse.value = null;
    try {
      await searchStore.run(projectId, text);
      const nextReferences = searchStore.results.slice(0, 8).map(toAgentReference);
      contextReferences.value = nextReferences;
      const response = await askAgent({
        projectId,
        question: text,
        references: nextReferences,
      });
      lastResponse.value = response;
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : String(error);
    } finally {
      asking.value = false;
    }
  }

  async function saveReport() {
    const response = lastResponse.value;
    if (!selectedProjectId.value || !response) return;

    savingReport.value = true;
    errorMessage.value = "";
    statusMessage.value = "";
    try {
      await libraryStore.createEntry({
        projectId: selectedProjectId.value,
        entryType: "report",
        title: reportTitle(),
        summary: question.value.trim(),
        body: buildReportBody(response),
        tags: ["agent", "report"],
        status: "active",
      });
      statusMessage.value = "已保存为本地报告";
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : String(error);
    } finally {
      savingReport.value = false;
    }
  }

  async function saveTask() {
    if (!selectedProjectId.value || !question.value.trim()) return;

    savingTask.value = true;
    errorMessage.value = "";
    statusMessage.value = "";
    try {
      await jobStore.createJob({
        projectId: selectedProjectId.value,
        providerKind: "openAiCompatible",
        jobType: "agentChat",
        inputSummary: question.value.trim(),
      });
      statusMessage.value = "已保存为任务";
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : String(error);
    } finally {
      savingTask.value = false;
    }
  }

  function toAgentReference(result: SearchReferenceInput): AgentAskReference {
    return {
      entityType: result.entityType,
      entityId: result.entityId,
      title: result.title,
      snippet: result.snippet,
      source: result.source,
      score: result.score,
    };
  }

  function reportTitle(): string {
    const text = question.value.trim();
    return text.length > 28 ? `Agent 问答：${text.slice(0, 28)}...` : `Agent 问答：${text}`;
  }

  function buildReportBody(response: AgentAskResponse): string {
    const sourceLines = response.references.length
      ? response.references
          .map(
            (reference, index) =>
              `${index + 1}. ${entityTypeLabel(reference.entityType)} ${reference.title} (${reference.entityType}:${reference.entityId}) - ${reference.snippet || "无摘要"}`,
          )
          .join("\n")
      : "无引用来源";
    return `# 问题\n${question.value.trim()}\n\n# 回答\n${response.answer}\n\n# 引用来源\n${sourceLines}`;
  }

  function formatScore(score: number): string {
    return score.toFixed(2);
  }

  return {
    answer,
    asking,
    canSave,
    canSubmit,
    errorMessage,
    formatScore,
    lastResponse,
    projectStore,
    question,
    references,
    saveReport,
    saveTask,
    savingReport,
    savingTask,
    selectedProjectId,
    semanticNotice,
    statusMessage,
    submitQuestion,
  };
}
