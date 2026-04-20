import {defineStore} from 'pinia'
import {ref} from 'vue'
import {invoke} from '@tauri-apps/api/core'

export const useActivityStore = defineStore('activity', () => {
    const activities = ref([])  // ActivityDetail[]
    const loading = ref(false)
    const error = ref('')

    async function fetchActivities() {
        loading.value = true
        error.value = ''
        try {
            activities.value = await invoke('get_activities')
        } catch (e) {
            error.value = String(e)
        } finally {
            loading.value = false
        }
    }

    async function createActivity(name) {
        await invoke('create_activity', {name})
        await fetchActivities()
    }

    async function updateActivity(id, name) {
        await invoke('update_activity', {id, name})
        await fetchActivities()
    }

    async function deleteActivity(id) {
        await invoke('delete_activity', {id})
        await fetchActivities()
    }

    return {activities, loading, error, fetchActivities, createActivity, updateActivity, deleteActivity}
})
