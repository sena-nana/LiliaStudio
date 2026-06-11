<template>
  <section class="view-stack">
    <header class="view-header">
      <div>
        <p class="eyebrow">图谱</p>
        <h1>关系邻域</h1>
      </div>
      <p v-if="libraryStore.error" class="status-note error">{{ libraryStore.error }}</p>
    </header>

    <div v-if="!projectId" class="empty-state">
      <h2>未选择项目</h2>
    </div>

    <template v-else>
      <section class="graph-toolbar">
        <label>
          关系类型
          <select v-model="relationTypeFilter">
            <option value="">全部关系</option>
            <option v-for="type in relationTypes" :key="type" :value="type">{{ type }}</option>
          </select>
        </label>
        <label>
          实体类型
          <select v-model="entityTypeFilter">
            <option value="">全部实体</option>
            <option v-for="type in entityTypes" :key="type" :value="type">{{ entityTypeLabel(type) }}</option>
          </select>
        </label>
        <button type="button" class="secondary-button" :disabled="!focusedNodeKey" @click="focusedNodeKey = ''">清除聚焦</button>
      </section>

      <div v-if="visibleEdges.length === 0" class="empty-state">
        <h2>暂无匹配关系</h2>
      </div>

      <div v-else class="graph-layout">
        <section class="graph-board" aria-label="关系图谱">
          <svg class="graph-edges" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
            <line
              v-for="edge in graphEdges"
              :key="edge.id"
              :x1="edge.source.x"
              :y1="edge.source.y"
              :x2="edge.target.x"
              :y2="edge.target.y"
              :class="{ muted: focusedNodeKey && edge.source.key !== focusedNodeKey && edge.target.key !== focusedNodeKey }"
            />
          </svg>

          <button
            v-for="node in graphNodes"
            :key="node.key"
            type="button"
            class="graph-node"
            :class="{ active: node.key === focusedNodeKey, muted: focusedNodeKey && !node.relatedToFocus }"
            :style="{ left: `${node.x}%`, top: `${node.y}%` }"
            @click="focusNode(node.key)"
            @dblclick="openRecord(node.entityType, node.entityId)"
          >
            <strong>{{ node.title }}</strong>
            <span>{{ entityTypeLabel(node.entityType) }} · {{ node.relationCount }} 关联</span>
          </button>
        </section>

        <aside class="graph-side">
          <section>
            <h2>真实关系</h2>
            <p>{{ visibleEdges.length }} 条边 · {{ graphNodes.length }} 个节点</p>
          </section>

          <section v-if="focusedNode">
            <h2>当前节点</h2>
            <p>{{ focusedNode.title }}</p>
            <p>{{ focusedNode.subtitle }}</p>
            <button type="button" class="secondary-button" @click="openRecord(focusedNode.entityType, focusedNode.entityId)">编辑对象</button>
          </section>

          <section>
            <h2>边</h2>
            <ul class="graph-edge-list">
              <li v-for="edge in visibleEdges" :key="edge.id">
                <button type="button" @click="openRecord('relation', edge.id)">
                  <strong>{{ edge.relationType }}</strong>
                  <span>{{ edgeLabel(edge) }}</span>
                </button>
              </li>
            </ul>
          </section>
        </aside>
      </div>
    </template>
  </section>
</template>

<script setup lang="ts">
import { entityTypeLabel } from "@/domain/displayLabels";
import { useRelationGraphWorkspace } from "@/composables/useRelationGraphWorkspace";

const {
  edgeLabel,
  entityTypeFilter,
  entityTypes,
  focusNode,
  focusedNode,
  focusedNodeKey,
  graphEdges,
  graphNodes,
  libraryStore,
  openRecord,
  projectId,
  relationTypeFilter,
  relationTypes,
  visibleEdges,
} = useRelationGraphWorkspace();
</script>
