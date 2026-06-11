<template>
  <aside class="object-inspector">
    <template v-if="selectedEntityRef && neighborhood">
      <header>
        <div>
          <p class="eyebrow">{{ entityTypeLabel(neighborhood.center.entityType) }}</p>
          <h2>{{ neighborhood.center.title }}</h2>
          <p>{{ neighborhood.center.subtitle || "未分类" }}</p>
        </div>
        <strong>{{ neighborhood.relationCount }}</strong>
      </header>

      <section>
        <h3>摘要</h3>
        <p>{{ neighborhood.center.summary || "暂无摘要" }}</p>
      </section>

      <section>
        <h3>已有关联</h3>
        <ul class="inspector-list">
          <li v-for="edge in neighborhood.edges" :key="edge.id">
            <button type="button" @click="$emit('selectEdgeNeighbor', edge)">
              <strong>{{ edge.relationType }}</strong>
              <span>{{ edgeLabel(edge) }}</span>
            </button>
          </li>
        </ul>
        <p v-if="neighborhood.edges.length === 0" class="muted">暂无已确认关系</p>
      </section>

      <section>
        <h3>待确认建议</h3>
        <ul class="inspector-list">
          <li v-for="suggestion in neighborhood.suggestions" :key="suggestionKey(suggestion)">
            <div class="suggestion-row">
              <div>
                <strong>{{ suggestion.relationType }} · {{ suggestion.strength }}</strong>
                <span>{{ suggestionLabel(suggestion) }} · {{ suggestion.reason }}</span>
              </div>
              <button type="button" class="secondary-button" @click="$emit('confirmSuggestion', suggestion)">确认</button>
            </div>
          </li>
        </ul>
        <p v-if="neighborhood.suggestions.length === 0" class="muted">暂无待确认建议</p>
      </section>

      <section>
        <h3>缺失项</h3>
        <div class="chip-list">
          <span v-for="item in neighborhood.missing" :key="item" class="status-chip">{{ item }}</span>
          <span v-if="neighborhood.missing.length === 0" class="status-chip ok">结构完整</span>
        </div>
      </section>

      <section>
        <h3>邻域预览</h3>
        <div class="neighborhood-map">
          <button
            v-for="node in neighborhood.nodes"
            :key="`${node.entityType}:${node.entityId}`"
            type="button"
            class="map-node"
            :class="{ active: node.entityType === neighborhood.center.entityType && node.entityId === neighborhood.center.entityId }"
            @click="$emit('selectEntity', node.entityType, node.entityId)"
          >
            <strong>{{ node.title }}</strong>
            <span>{{ entityTypeLabel(node.entityType) }}</span>
          </button>
        </div>
      </section>
    </template>

    <template v-else-if="selected.kind === 'relation' && selectedRelation">
      <header>
        <div>
          <p class="eyebrow">关系</p>
          <h2>{{ selectedRelation.relationType }}</h2>
          <p>{{ selectedRelation.confidence.toFixed(2) }} 置信度</p>
        </div>
      </header>
      <section>
        <h3>连接</h3>
        <p>{{ entityLabel(selectedRelation.source, entityOptions) }}</p>
        <p>{{ entityLabel(selectedRelation.target, entityOptions) }}</p>
      </section>
    </template>

    <div v-else class="empty-state compact">
      <h2>未选择</h2>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { entityTypeLabel } from "@/domain/displayLabels";
import { entityLabel, type LibraryEntityOption, type LibrarySelection } from "@/domain/libraryWorkspace";
import type { EntityRef, Relation, RelationNeighborhood, RelationSuggestion } from "@/types/library";

const props = defineProps<{
  entityOptions: LibraryEntityOption[];
  neighborhood: RelationNeighborhood | null;
  selected: LibrarySelection;
  selectedEntityRef: EntityRef | null;
  selectedRelation: Relation | null;
}>();

defineEmits<{
  confirmSuggestion: [suggestion: RelationSuggestion];
  selectEdgeNeighbor: [edge: Relation];
  selectEntity: [entityType: string, entityId: string];
}>();

function edgeLabel(edge: Relation): string {
  return `${entityLabel(edge.source, props.entityOptions)} -> ${entityLabel(edge.target, props.entityOptions)}`;
}

function suggestionLabel(suggestion: RelationSuggestion): string {
  return `${entityLabel(suggestion.source, props.entityOptions)} -> ${entityLabel(suggestion.target, props.entityOptions)}`;
}

function suggestionKey(suggestion: RelationSuggestion): string {
  return `${suggestion.source.entityType}:${suggestion.source.entityId}:${suggestion.target.entityType}:${suggestion.target.entityId}:${suggestion.relationType}`;
}
</script>
