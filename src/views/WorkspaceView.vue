<script setup>
import { useProjectStore } from '../stores/project'
const project = useProjectStore()
</script>

<template>
  <div class="p-8">
    <h1 class="text-2xl font-bold mb-2">메인 워크스페이스</h1>
    <p class="text-gray-500 text-sm">{{ project.filePath }}</p>
  </div>
</template>
