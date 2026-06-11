<template>
  <LibraryEditorShell :title="title" :has-selection="Boolean(selected.id)" @delete="$emit('delete')" @save="$emit('save')">
    <div v-if="selected.kind === 'entry'" class="editor-form">
      <label>标题<input v-model="forms.entry.title" /></label>
      <label>类型<input v-model="forms.entry.entryType" /></label>
      <label>状态<input v-model="forms.entry.status" /></label>
      <label>标签<input v-model="forms.entryTagsText.value" placeholder="标签" /></label>
      <div class="wide rich-text-field">
        <span class="field-label">摘要</span>
        <Suspense>
          <RichTextEditor v-model="forms.entry.summary" />
          <template #fallback><div class="rich-text-loading">编辑器加载中</div></template>
        </Suspense>
      </div>
      <div class="wide rich-text-field">
        <span class="field-label">正文</span>
        <Suspense>
          <RichTextEditor v-model="forms.entry.body" />
          <template #fallback><div class="rich-text-loading">编辑器加载中</div></template>
        </Suspense>
      </div>
    </div>

    <div v-else-if="selected.kind === 'character'" class="editor-form">
      <label>姓名<input v-model="forms.character.name" /></label>
      <label>阵营<input v-model="forms.character.faction" /></label>
      <label>别名<input v-model="forms.characterAliasesText.value" placeholder="别名" /></label>
      <label>标签<input v-model="forms.characterTagsText.value" placeholder="标签" /></label>
      <label class="wide">摘要<textarea v-model="forms.character.summary" rows="3" /></label>
      <label>外貌<textarea v-model="forms.character.appearance" rows="4" /></label>
      <label>目标<textarea v-model="forms.character.goals" rows="4" /></label>
      <label>动机<textarea v-model="forms.character.motivations" rows="4" /></label>
      <label>恐惧<textarea v-model="forms.character.fears" rows="4" /></label>
    </div>

    <div v-else-if="selected.kind === 'event'" class="editor-form">
      <label>标题<input v-model="forms.event.title" /></label>
      <label>时间<input v-model="forms.event.timeLabel" /></label>
      <label>排序<input v-model.number="forms.event.sortKey" type="number" /></label>
      <label>地点<input v-model="forms.event.location" /></label>
      <label>开始<input v-model="forms.event.startLabel" /></label>
      <label>结束<input v-model="forms.event.endLabel" /></label>
      <label>重要度<input v-model.number="forms.event.importance" min="0" max="10" type="number" /></label>
      <label>标签<input v-model="forms.eventTagsText.value" placeholder="标签" /></label>
      <label class="wide">描述<textarea v-model="forms.event.description" rows="4" /></label>
      <label class="wide">结果<textarea v-model="forms.event.outcome" rows="4" /></label>
    </div>

    <div v-else-if="selected.kind === 'axiom'" class="editor-form">
      <label>主体<input v-model="forms.axiom.subject" /></label>
      <label>谓词<input v-model="forms.axiom.predicate" /></label>
      <label>对象<input v-model="forms.axiom.object" /></label>
      <label>确定性<input v-model.number="forms.axiom.certainty" min="0" max="1" step="0.05" type="number" /></label>
      <label>时间范围<input v-model="forms.axiom.scopeTime" /></label>
      <label>地点范围<input v-model="forms.axiom.scopeLocation" /></label>
      <label>来源类型<input v-model="forms.sourceEntityTypeText.value" /></label>
      <label>来源标识<input v-model="forms.sourceEntityIdText.value" /></label>
      <label>标签<input v-model="forms.axiomTagsText.value" placeholder="标签" /></label>
      <label class="wide">自然语言<textarea v-model="forms.axiom.naturalLanguage" rows="5" /></label>
    </div>

    <div v-else-if="selected.kind === 'relation'" class="editor-form">
      <label>
        来源
        <select v-model="forms.relationSourceKey.value">
          <option v-for="option in entityOptions" :key="option.key" :value="option.key">
            {{ option.label }}
          </option>
        </select>
      </label>
      <label>
        目标
        <select v-model="forms.relationTargetKey.value">
          <option v-for="option in entityOptions" :key="option.key" :value="option.key">
            {{ option.label }}
          </option>
        </select>
      </label>
      <label>
        关系类型
        <input v-model="forms.relation.relationType" list="relation-type-presets" />
      </label>
      <datalist id="relation-type-presets">
        <option v-for="type in relationTypes" :key="type" :value="type" />
      </datalist>
      <label>置信度<input v-model.number="forms.relation.confidence" min="0" max="1" step="0.05" type="number" /></label>
      <label class="toggle-row"><input v-model="forms.relation.directed" type="checkbox" />有方向</label>
      <label class="wide">描述<textarea v-model="forms.relation.description" rows="5" /></label>
    </div>
  </LibraryEditorShell>
</template>

<script setup lang="ts">
import { defineAsyncComponent } from "vue";
import type { LibraryRecordForms } from "@/composables/useLibraryRecordForms";
import type { LibraryEntityOption, LibrarySelection } from "@/domain/libraryWorkspace";
import LibraryEditorShell from "@/components/library/LibraryEditorShell.vue";

const RichTextEditor = defineAsyncComponent(() => import("@/components/RichTextEditor.vue"));

defineProps<{
  entityOptions: LibraryEntityOption[];
  forms: LibraryRecordForms;
  relationTypes: string[];
  selected: LibrarySelection;
  title: string;
}>();

defineEmits<{
  delete: [];
  save: [];
}>();
</script>
