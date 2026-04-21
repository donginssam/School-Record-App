<script setup>
import {BookOpen, ChevronRight, Layers, PenLine, Upload, Users} from 'lucide-vue-next'

const emit = defineEmits(['select'])

const steps = [
  {
    num: 1,
    icon: Users,
    title: '학생 등록',
    desc: '학년·반·번호·이름을 등록합니다. 엑셀 파일로 명렬표를 일괄 불러올 수 있습니다.',
    section: 'student',
    color: '#3b82f6',
    bg: 'rgba(59,130,246,0.07)',
    border: 'rgba(59,130,246,0.25)',
  },
  {
    num: 2,
    icon: BookOpen,
    title: 'Activity 생성',
    desc: '수업·프로젝트·동아리 등 생기부에 들어갈 각 활동 단위를 만듭니다.',
    section: 'activity',
    color: '#818cf8',
    bg: 'rgba(129,140,248,0.07)',
    border: 'rgba(129,140,248,0.25)',
  },
  {
    num: 3,
    icon: Layers,
    title: 'Area 구성',
    desc: '여러 Activity를 묶어 진로활동·자율활동·동아리 등 생기부 영역을 완성합니다.',
    section: 'area',
    color: '#a855f7',
    bg: 'rgba(168,85,247,0.07)',
    border: 'rgba(168,85,247,0.25)',
  },
  {
    num: 4,
    icon: PenLine,
    title: '기록 작성',
    desc: '학생별·활동별 생기부 문장을 셀 단위로 입력합니다. 바이트 제한을 자동으로 표시합니다.',
    section: 'record',
    color: '#f59e0b',
    bg: 'rgba(245,158,11,0.07)',
    border: 'rgba(245,158,11,0.25)',
  },
  {
    num: 5,
    icon: Upload,
    title: '내보내기',
    desc: '완성된 생기부 문장을 엑셀 파일로 추출합니다. A·B·C 세 가지 형식을 지원합니다.',
    section: 'export',
    color: '#10b981',
    bg: 'rgba(16,185,129,0.07)',
    border: 'rgba(16,185,129,0.25)',
  },
]

const exampleActivities = [
  {name: '전공탐색 퀴즈 대회', desc: '학생 주도 참여'},
  {name: '전공 도서 독후감', desc: '독서 기록 활동'},
  {name: '진로 상담 프로그램', desc: '개별 상담 참여'},
  {name: '직업인 특강 청취', desc: '외부 강사 연계'},
]
</script>

<template>
  <div class="section">
    <div class="section-body">

      <!-- 히어로 -->
      <div class="hero">
        <div class="hero-eyebrow">
          <span class="eyebrow-badge">생기부</span>
          학교생활기록부 작성 도우미
        </div>
        <h1 class="hero-title">5단계로 완성하는<br>체계적인 생기부 관리</h1>
        <p class="hero-sub">
          학생 명렬표 등록부터 Activity 조합, 영역 구성, 기록 작성, 내보내기까지<br>
          아래 순서대로 진행하면 학생별 생활기록부 문장을 완성할 수 있습니다.
        </p>
      </div>

      <!-- 워크플로 스텝 -->
      <div class="steps">
        <div
            v-for="step in steps"
            :key="step.num"
            class="step-card"
            :class="{ 'step-card--wide': step.num === 5 }"
            :style="{ '--c': step.color, '--bg': step.bg, '--bd': step.border }"
            @click="emit('select', step.section)"
        >
          <div class="step-left">
            <div class="step-num">{{ step.num }}</div>
            <component :is="step.icon" :size="28" class="step-icon"/>
          </div>
          <div class="step-middle">
            <div class="step-name">{{ step.title }}</div>
            <div class="step-desc">{{ step.desc }}</div>
          </div>
          <button class="step-btn" @click.stop="emit('select', step.section)">
            이동하기
            <ChevronRight :size="16"/>
          </button>
        </div>
      </div>

      <!-- 구조 설명 -->
      <div class="structure">
        <div class="structure-header">
          <h2 class="structure-title">이 프로그램은 어떻게 작동하나요?</h2>
          <p class="structure-sub">
            생기부의 각 항목은 <strong>영역(Area)</strong>이라고 부릅니다.
            예를 들어 <strong>진로활동</strong>이 하나의 영역입니다.<br>
            그 안에 학생이 실제로 참여한 개별 활동들, 즉 <strong>Activity</strong>를 여러 개 담아 하나의 영역을 완성합니다.
          </p>
        </div>

        <!-- Area 박스 다이어그램 -->
        <div class="area-box">
          <div class="area-box-header">
            <span class="area-tag">Area</span>
            <span class="area-box-name">진로활동</span>
            <span class="area-box-limit">최대 1,500 byte</span>
          </div>

          <div class="area-box-desc">
            아래 Activity들의 기록이 합쳐져 이 영역 하나를 구성합니다.
          </div>

          <div class="activities-grid">
            <div
                v-for="act in exampleActivities"
                :key="act.name"
                class="activity-card"
            >
              <div class="activity-dot"></div>
              <div class="activity-info">
                <div class="activity-name">{{ act.name }}</div>
                <div class="activity-sub">{{ act.desc }}</div>
              </div>
            </div>

            <!-- 추가 가능 암시 카드 -->
            <div class="activity-card activity-card--more">
              <div class="activity-more-icon">＋</div>
              <div class="activity-info">
                <div class="activity-name" style="color:#5a7090;">활동 더 추가 가능</div>
                <div class="activity-sub">원하는 만큼</div>
              </div>
            </div>
          </div>

          <div class="area-box-footer">
            각 Activity마다 학생별로 기록을 작성하면, 합산 문장이 <strong>진로활동</strong> 항목으로 완성됩니다.
          </div>
        </div>
      </div>

    </div>
  </div>
</template>

<style scoped>
.section {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  box-sizing: border-box;
}

.section-body {
  flex: 1;
  overflow-y: auto;
  padding: 44px 44px 60px;
  display: flex;
  flex-direction: column;
  gap: 52px;
}

/* ── 히어로 ─────────────────────────────────────────────────── */
.hero {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.hero-eyebrow {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 15px;
  font-weight: 500;
  color: #93b8d8;
}

.eyebrow-badge {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.06em;
  color: #fbbf24;
  background: rgba(251, 191, 36, 0.12);
  border: 1px solid rgba(251, 191, 36, 0.28);
  border-radius: 5px;
  padding: 2px 8px;
}

.hero-title {
  font-size: 38px;
  font-weight: 800;
  color: #eef2f8;
  margin: 0;
  line-height: 1.28;
  letter-spacing: -0.02em;
}

.hero-sub {
  font-size: 16px;
  color: #93b8d8;
  margin: 0;
  line-height: 1.8;
}

/* ── 워크플로 스텝 ───────────────────────────────────────────── */
.steps {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
}

.step-card {
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 22px 24px;
  background: var(--bg);
  border: 1px solid var(--bd);
  border-radius: 16px;
  cursor: pointer;
  transition: border-color 0.15s, box-shadow 0.15s, transform 0.15s;
}

.step-card--wide {
  grid-column: 1 / -1;
}

.step-card:hover {
  border-color: var(--c);
  box-shadow: 0 4px 24px color-mix(in srgb, var(--c) 18%, transparent);
  transform: translateY(-2px);
}

.step-left {
  display: flex;
  align-items: center;
  gap: 14px;
  flex-shrink: 0;
}

.step-num {
  width: 50px;
  height: 50px;
  border-radius: 50%;
  background: color-mix(in srgb, var(--c) 14%, transparent);
  border: 2px solid color-mix(in srgb, var(--c) 35%, transparent);
  color: var(--c);
  font-size: 20px;
  font-weight: 800;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.step-icon {
  color: var(--c);
  opacity: 0.85;
}

.step-middle {
  flex: 1;
  min-width: 0;
}

.step-name {
  font-size: 18px;
  font-weight: 700;
  color: #eef2f8;
  margin-bottom: 5px;
}

.step-desc {
  font-size: 14px;
  color: #8bb2cc;
  line-height: 1.6;
}

.step-btn {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 8px 16px;
  border-radius: 10px;
  background: color-mix(in srgb, var(--c) 12%, transparent);
  border: 1px solid color-mix(in srgb, var(--c) 30%, transparent);
  color: var(--c);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  flex-shrink: 0;
  white-space: nowrap;
  transition: background 0.15s, border-color 0.15s;
}

.step-btn:hover {
  background: color-mix(in srgb, var(--c) 22%, transparent);
  border-color: var(--c);
}

/* ── 구조 설명 ───────────────────────────────────────────────── */
.structure {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.structure-header {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.structure-title {
  font-size: 22px;
  font-weight: 700;
  color: #eef2f8;
  margin: 0;
}

.structure-sub {
  font-size: 15px;
  color: #93b8d8;
  line-height: 1.8;
  margin: 0;
}

.structure-sub strong {
  color: #c8d8f0;
  font-weight: 700;
}

.structure-sub em {
  font-style: normal;
  color: #c8d8f0;
}

/* ── Area 박스 다이어그램 ────────────────────────────────────── */
.area-box {
  border: 2px solid rgba(168, 85, 247, 0.35);
  border-radius: 20px;
  background: rgba(168, 85, 247, 0.04);
  padding: 28px 32px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.area-box-header {
  display: flex;
  align-items: center;
  gap: 14px;
}

.area-tag {
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.06em;
  color: #a855f7;
  background: rgba(168, 85, 247, 0.16);
  border: 1px solid rgba(168, 85, 247, 0.35);
  border-radius: 6px;
  padding: 3px 10px;
}

.area-box-name {
  font-size: 22px;
  font-weight: 800;
  color: #eef2f8;
}

.area-box-limit {
  font-size: 13px;
  color: #718fad;
  margin-left: auto;
  border: 1px solid #30395c;
  border-radius: 6px;
  padding: 3px 10px;
}

.area-box-desc {
  font-size: 14px;
  color: #6a8aaa;
  padding-bottom: 4px;
  border-bottom: 1px solid #1a2035;
}

.activities-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 12px;
}

.activity-card {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px 18px;
  background: #0d1220;
  border: 1px solid rgba(129, 140, 248, 0.25);
  border-radius: 12px;
  transition: border-color 0.15s;
}

.activity-card:hover {
  border-color: rgba(129, 140, 248, 0.5);
}

.activity-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #818cf8;
  flex-shrink: 0;
  opacity: 0.75;
}

.activity-card--more {
  border-style: dashed;
  border-color: #1e2a45;
  background: transparent;
}

.activity-more-icon {
  width: 10px;
  text-align: center;
  font-size: 16px;
  color: #3a4a6b;
  flex-shrink: 0;
}

.activity-info {
  min-width: 0;
}

.activity-name {
  font-size: 15px;
  font-weight: 600;
  color: #d0e0f0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.activity-sub {
  font-size: 12px;
  color: #5a7090;
  margin-top: 2px;
}

.area-box-footer {
  font-size: 14px;
  color: #6a8aaa;
  padding-top: 4px;
  border-top: 1px solid #1a2035;
  line-height: 1.7;
}

.area-box-footer strong {
  color: #a880f0;
  font-weight: 600;
}
</style>
