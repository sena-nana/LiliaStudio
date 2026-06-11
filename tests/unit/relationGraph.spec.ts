import { describe, expect, it } from "vitest";
import {
  buildGraphEdges,
  buildGraphEntityRefs,
  buildGraphNodes,
  edgeLabel,
  layoutPoint,
  uniqueSorted,
} from "@/domain/relationGraph";
import { buildEntityOptions } from "@/domain/libraryWorkspace";
import type { LibraryProjectSnapshot, Relation } from "@/types/library";

const relation: Relation = {
  id: "relation_1",
  projectId: "project_1",
  source: { entityType: "character", entityId: "character_1" },
  target: { entityType: "event", entityId: "event_1" },
  relationType: "参与事件",
  description: "",
  confidence: 1,
  directed: true,
  createdAt: "",
  updatedAt: "",
  deletedAt: null,
};

const collections: Pick<LibraryProjectSnapshot, "entries" | "characters" | "events" | "axioms"> = {
  entries: [],
  characters: [
    {
      id: "character_1",
      projectId: "project_1",
      name: "椎名",
      aliases: [],
      summary: "",
      appearance: "",
      goals: "",
      motivations: "",
      fears: "",
      faction: "北境",
      tags: [],
      createdAt: "",
      updatedAt: "",
      deletedAt: null,
    },
  ],
  events: [
    {
      id: "event_1",
      projectId: "project_1",
      title: "围城战",
      description: "",
      timeLabel: "冬季",
      sortKey: 1,
      startLabel: "",
      endLabel: "",
      location: "",
      importance: 4,
      outcome: "",
      tags: [],
      createdAt: "",
      updatedAt: "",
      deletedAt: null,
    },
  ],
  axioms: [],
};

describe("relation graph helpers", () => {
  it("builds graph nodes and edges from visible relations", () => {
    const options = buildEntityOptions(collections);
    const nodes = buildGraphNodes([relation], collections, options, "character:character_1");
    const edges = buildGraphEdges([relation], nodes);

    expect(buildGraphEntityRefs([relation])).toEqual([relation.source, relation.target]);
    expect(nodes).toMatchObject([
      { key: "character:character_1", title: "椎名", subtitle: "北境", relatedToFocus: true },
      { key: "event:event_1", title: "围城战", subtitle: "冬季", relatedToFocus: true },
    ]);
    expect(edges[0]).toMatchObject({ id: "relation_1", source: nodes[0], target: nodes[1] });
    expect(edgeLabel(relation, options)).toBe("角色：椎名 -> 事件：围城战");
  });

  it("keeps layout and sorting deterministic", () => {
    expect(layoutPoint(0, 1)).toEqual({ x: 50, y: 50 });
    expect(uniqueSorted(["关系", "参与事件", "关系"])).toEqual(["参与事件", "关系"]);
  });
});
