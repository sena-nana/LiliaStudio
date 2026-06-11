import { entityTypeLabel } from "@/domain/displayLabels";
import type { AgentAskReference, AgentAskResponse } from "@/types/ai";

export interface SearchReferenceInput {
  entityType: string;
  entityId: string;
  title: string;
  snippet: string;
  source: string;
  score: number;
}

export function toAgentReference(result: SearchReferenceInput): AgentAskReference {
  return {
    entityType: result.entityType,
    entityId: result.entityId,
    title: result.title,
    snippet: result.snippet,
    source: result.source,
    score: result.score,
  };
}

export function agentReportTitle(question: string): string {
  const text = question.trim();
  return text.length > 28 ? `Agent 问答：${text.slice(0, 28)}...` : `Agent 问答：${text}`;
}

export function buildAgentReportBody(question: string, response: AgentAskResponse): string {
  const sourceLines = response.references.length
    ? response.references
        .map(
          (reference, index) =>
            `${index + 1}. ${entityTypeLabel(reference.entityType)} ${reference.title} (${reference.entityType}:${reference.entityId}) - ${reference.snippet || "无摘要"}`,
        )
        .join("\n")
    : "无引用来源";
  return `# 问题\n${question.trim()}\n\n# 回答\n${response.answer}\n\n# 引用来源\n${sourceLines}`;
}

export function formatReferenceScore(score: number): string {
  return score.toFixed(2);
}
