import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useProjectStore = defineStore('project', () => {
  const isOpen = ref(false)
  const filePath = ref('')

  function setProject(path) {
    filePath.value = path
    isOpen.value = true
  }

  function closeProject() {
    filePath.value = ''
    isOpen.value = false
  }

  return { isOpen, filePath, setProject, closeProject }
})
