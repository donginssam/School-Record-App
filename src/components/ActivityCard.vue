<script setup>
import {computed} from 'vue'

const props = defineProps({
  activity: {type: Object, required: true},
})

const emit = defineEmits(['edit'])

const CHIP_MAX = 4

const visibleAreas = computed(() =>
    props.activity.areas.slice(0, CHIP_MAX)
)

const hiddenCount = computed(() =>
    Math.max(0, props.activity.areas.length - CHIP_MAX)
)
</script>

<template>
  <div class="card" @click="emit('edit', activity)">
    <div class="card-header">
      <h3 class="activity-name">{{ activity.name }}</h3>
      <span
          class="area-count-badge"
          :class="activity.areas.length === 0 ? 'area-count-badge--empty' : ''"
      >
        {{ activity.areas.length === 0 ? '미배정' : `${activity.areas.length}개 영역` }}
      </span>
    </div>

    <!-- 소속 영역 칩 -->
    <div v-if="activity.areas.length > 0" class="chip-row">
      <span v-for="area in visibleAreas" :key="area.id" class="chip">
        {{ area.name }}
      </span>
      <span v-if="hiddenCount > 0" class="chip chip--more">+{{ hiddenCount }}개 더</span>
    </div>
    <p v-else class="no-area">아직 어느 영역에도 포함되지 않았습니다.</p>

    <div class="edit-hint">클릭하여 편집</div>
  </div>
</template>

<style scoped>
.card {
  background-color: #0e1220;
  border: 1px solid #1a2035;
  border-radius: 16px;
  padding: 22px;
  cursor: pointer;
  transition: border-color 0.15s, box-shadow 0.15s;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.card:hover {
  border-color: rgba(59, 91, 219, 0.5);
  box-shadow: 0 4px 24px rgba(59, 91, 219, 0.08);
}

.card-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.activity-name {
  font-size: 18px;
  font-weight: 700;
  color: #e2e8f0;
  margin: 0;
  line-height: 1.3;
}

.area-count-badge {
  font-size: 13px;
  font-weight: 600;
  color: #7ba8f0;
  background-color: rgba(59, 91, 219, 0.12);
  border: 1px solid rgba(59, 91, 219, 0.25);
  border-radius: 6px;
  padding: 3px 8px;
  white-space: nowrap;
  flex-shrink: 0;
}

.area-count-badge--empty {
  color: #fbbf24;
  background-color: rgba(251, 191, 36, 0.1);
  border-color: rgba(251, 191, 36, 0.3);
}

.chip-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.chip {
  font-size: 13px;
  color: #93c5fd;
  background-color: rgba(59, 91, 219, 0.1);
  border: 1px solid rgba(59, 91, 219, 0.2);
  border-radius: 20px;
  padding: 3px 10px;
}

.chip--more {
  color: #7ba3d4;
  background-color: rgba(255, 255, 255, 0.04);
  border-color: #2a3a58;
}

.no-area {
  font-size: 14px;
  color: #7ba3d4;
  margin: 0;
}

.edit-hint {
  font-size: 12px;
  color: #3d5580;
  text-align: right;
  margin-top: auto;
}

.card:hover .edit-hint {
  color: #5a7aaa;
}
</style>
