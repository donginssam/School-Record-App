<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { save, open } from '@tauri-apps/plugin-dialog'
import { useProjectStore } from '../stores/project'

const router = useRouter()
const project = useProjectStore()
const error = ref('')

async function handleNew() {
  error.value = ''
  const path = await save({
    title: '새 학생부 파일 위치 선택',
    defaultPath: 'school_record.db',
    filters: [{ name: 'SQLite DB', extensions: ['db'] }],
  })
  if (!path) return

  try {
    await invoke('new_project', { path })
    project.setProject(path)
    router.push('/workspace')
  } catch (e) {
    error.value = `파일 생성 실패: ${e}`
  }
}

async function handleOpen() {
  error.value = ''
  const path = await open({
    title: '기존 학생부 파일 선택',
    filters: [{ name: 'SQLite DB', extensions: ['db'] }],
    multiple: false,
  })
  if (!path) return

  try {
    await invoke('open_project', { path })
    project.setProject(path)
    router.push('/workspace')
  } catch (e) {
    error.value = `파일 열기 실패: ${e}`
  }
}
</script>

<template>
  <div class="flex flex-col items-center justify-center min-h-screen gap-6 bg-gray-50">
    <h1 class="text-3xl font-bold text-gray-800">학생부 관리 시스템</h1>

    <div class="flex flex-col gap-3 w-72">
      <button
        @click="handleNew"
        class="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition"
      >
        새로운 학생부 작성하기
      </button>
      <button
        @click="handleOpen"
        class="px-6 py-3 bg-green-600 text-white rounded-lg hover:bg-green-700 transition"
      >
        기존 학생부 이어쓰기
      </button>
      <button
        class="px-6 py-3 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition"
      >
        사용법 안내
      </button>
    </div>

    <p v-if="error" class="text-red-500 text-sm">{{ error }}</p>
  </div>
</template>
