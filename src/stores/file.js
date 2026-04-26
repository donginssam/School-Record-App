import {defineStore} from 'pinia'
import {ref} from 'vue'
import {invoke} from '@tauri-apps/api/core'

export const useFileStore = defineStore('file', () => {
    const loading = ref(false)
    const error = ref('')

    async function writeBytesFile(path, data) {
        loading.value = true
        error.value = ''
        try {
            await invoke('write_bytes_file', {path, data})
        } catch (e) {
            error.value = String(e)
            throw e
        } finally {
            loading.value = false
        }
    }

    return {loading, error, writeBytesFile}
})
