import {defineStore} from 'pinia'
import {ref} from 'vue'
import {invoke} from '@tauri-apps/api/core'

export const useRecordStore = defineStore('record', () => {
    const gridData = ref(null)
    const loading = ref(false)
    const error = ref('')

    async function fetchAreaGrid(areaId) {
        loading.value = true
        error.value = ''
        try {
            gridData.value = await invoke('get_area_grid', {areaId})
        } catch (e) {
            error.value = String(e)
            gridData.value = null
            throw e
        } finally {
            loading.value = false
        }
    }

    async function upsertRecord(activityId, studentId, content) {
        await invoke('upsert_record', {activityId, studentId, content})
    }

    return {gridData, loading, error, fetchAreaGrid, upsertRecord}
})
