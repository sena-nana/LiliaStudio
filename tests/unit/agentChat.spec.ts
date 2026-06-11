import { describe, expect, it } from "vitest";
import {
  agentReportTitle,
  buildAgentReportBody,
  formatReferenceScore,
  toAgentReference,
} from "@/domain/agentChat";
import type { AgentAskResponse } from "@/types/ai";

describe("agent chat helpers", () => {
  it("maps search results into agent references", () => {
    expect(
      toAgentReference({
        entityType: "entry",
        entityId: "entry_1",
        title: "月光阔剑",
        snippet: "潮汐能武器",
        source: "keyword",
        score: 2,
      }),
    ).toEqual({
      entityType: "entry",
      entityId: "entry_1",
      title: "月光阔剑",
      snippet: "潮汐能武器",
      source: "keyword",
      score: 2,
    });
  });

  it("builds report title and markdown body from the latest response", () => {
    const response: AgentAskResponse = {
      answer: "主要风险是补给不稳定。",
      providerKind: "openAiCompatible",
      model: "story-chat",
      references: [
        {
          entityType: "entry",
          entityId: "entry_1",
          title: "月光阔剑",
          snippet: "潮汐能武器",
          source: "keyword",
          score: 2,
        },
      ],
    };

    expect(agentReportTitle("月光阔剑有什么风险？")).toBe("Agent 问答：月光阔剑有什么风险？");
    expect(buildAgentReportBody(" 月光阔剑有什么风险？ ", response)).toContain(
      "1. 词条 月光阔剑 (entry:entry_1) - 潮汐能武器",
    );
    expect(formatReferenceScore(0.785)).toBe("0.79");
  });
});
