import {defineStore} from 'pinia'
import {ref} from 'vue'
import {invoke} from '@tauri-apps/api/core'

export const useActivityStore = defineStore('activity', () => {
    const activities = ref([])

    async function fetchActivities() {
        activities.value = await invoke('get_activities')
    }

    return {activities, fetchActivities}
})
