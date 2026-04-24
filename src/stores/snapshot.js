import {defineStore} from 'pinia'
import {invoke} from '@tauri-apps/api/core'

export const useSnapshotStore = defineStore('snapshot', () => {
    async function fetchSnapshots() {
        return await invoke('get_snapshots')
    }

    async function createSnapshot(memo) {
        return await invoke('create_snapshot', {memo: memo || null})
    }

    async function restoreSnapshot(snapshotId) {
        await invoke('restore_snapshot', {snapshotId})
    }

    return {fetchSnapshots, createSnapshot, restoreSnapshot}
})
