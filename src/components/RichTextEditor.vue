<template>
  <div class="rich-text-editor">
    <div class="rich-text-toolbar" aria-label="富文本工具栏">
      <button
        v-for="action in actions"
        :key="action.id"
        type="button"
        :class="{ active: action.active() }"
        :disabled="!editor || action.disabled()"
        :title="action.label"
        @click="action.run"
      >
        <component :is="action.icon" aria-hidden="true" />
      </button>
    </div>
    <EditorContent class="rich-text-surface" :editor="editor" />
  </div>
</template>

<script setup lang="ts">
import { computed, watch } from 'vue'
import { EditorContent, useEditor } from '@tiptap/vue-3'
import StarterKit from '@tiptap/starter-kit'
import {
  Bold,
  Heading2,
  Italic,
  List,
  ListOrdered,
  Quote,
  Redo2,
  Undo2,
} from 'lucide-vue-next'
import { normalizeRichTextContent, parseRichTextContent } from './richTextContent'

const props = defineProps<{
  modelValue: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const editor = useEditor({
  content: parseRichTextContent(props.modelValue),
  extensions: [StarterKit],
  editorProps: {
    attributes: {
      class: 'rich-text-content',
    },
  },
  onUpdate: ({ editor }) => {
    emit('update:modelValue', JSON.stringify(editor.getJSON()))
  },
})

watch(
  () => props.modelValue,
  (value) => {
    if (!editor.value) return
    const next = normalizeRichTextContent(value)
    const current = JSON.stringify(editor.value.getJSON())
    if (next !== current) {
      editor.value.commands.setContent(JSON.parse(next), { emitUpdate: false })
    }
  },
)

const actions = computed(() => [
  {
    id: 'bold',
    label: '加粗',
    icon: Bold,
    active: () => Boolean(editor.value?.isActive('bold')),
    disabled: () => !editor.value?.can().chain().focus().toggleBold().run(),
    run: () => editor.value?.chain().focus().toggleBold().run(),
  },
  {
    id: 'italic',
    label: '斜体',
    icon: Italic,
    active: () => Boolean(editor.value?.isActive('italic')),
    disabled: () => !editor.value?.can().chain().focus().toggleItalic().run(),
    run: () => editor.value?.chain().focus().toggleItalic().run(),
  },
  {
    id: 'heading',
    label: '二级标题',
    icon: Heading2,
    active: () => Boolean(editor.value?.isActive('heading', { level: 2 })),
    disabled: () => false,
    run: () => editor.value?.chain().focus().toggleHeading({ level: 2 }).run(),
  },
  {
    id: 'bullet-list',
    label: '无序列表',
    icon: List,
    active: () => Boolean(editor.value?.isActive('bulletList')),
    disabled: () => false,
    run: () => editor.value?.chain().focus().toggleBulletList().run(),
  },
  {
    id: 'ordered-list',
    label: '有序列表',
    icon: ListOrdered,
    active: () => Boolean(editor.value?.isActive('orderedList')),
    disabled: () => false,
    run: () => editor.value?.chain().focus().toggleOrderedList().run(),
  },
  {
    id: 'blockquote',
    label: '引用',
    icon: Quote,
    active: () => Boolean(editor.value?.isActive('blockquote')),
    disabled: () => false,
    run: () => editor.value?.chain().focus().toggleBlockquote().run(),
  },
  {
    id: 'undo',
    label: '撤销',
    icon: Undo2,
    active: () => false,
    disabled: () => !editor.value?.can().undo(),
    run: () => editor.value?.chain().focus().undo().run(),
  },
  {
    id: 'redo',
    label: '重做',
    icon: Redo2,
    active: () => false,
    disabled: () => !editor.value?.can().redo(),
    run: () => editor.value?.chain().focus().redo().run(),
  },
])
</script>
