# PROMPT TEMPLATES (Vue + Tauri)

```

TASK
<작업을 한 줄로 명확히 정의>

SCOPE
<대상 파일/디렉토리 명시 (glob 권장)>
- include:
- exclude:

RULES
- 구체적인 변환/수정 기준
- 예외 조건
- 유지해야 할 것

VALIDATION
- 테스트 / lint / 타입 체크 등
- 실패 시 처리 방식

OUTPUT
- 요약만 제공
- 변경 파일 수
- 핵심 변경 내용
- 이슈만 보고

```

이 파일은 반복 작업을 위한 표준 프롬프트 템플릿 모음이다.
각 템플릿은 sub-agent 실행을 전제로 설계되었으며, 반드시 SCOPE를 제한해서 사용한다.

---

## 0. BASE TEMPLATE

TASK
<one-line objective>

SCOPE
- include:
- exclude:

RULES
- transformation rules
- constraints
- must keep behavior

VALIDATION
- build / type / test
- fix only failing parts

OUTPUT
- summary only
- files changed
- key changes
- issues only

---

## 1. COMPONENT → STORE 구조 강제

TASK
Remove direct invoke() usage from Vue components

SCOPE
- include: src/components/**/*.vue

RULES
- move all invoke() calls to Pinia store
- create store actions if needed
- component handles UI only
- preserve existing behavior

VALIDATION
- no invoke() in components
- app builds successfully

OUTPUT
- summary only
- files updated
- new store actions
- remaining violations

---

## 2. STORE 표준화 (Pinia)

TASK
Standardize Pinia stores

SCOPE
- include: src/stores/*.ts

RULES
- async logic must use try/catch
- manage loading and error state
- keep state structure unchanged
- invoke() only inside store

VALIDATION
- no type errors
- no runtime errors

OUTPUT
- summary only
- stores updated
- applied patterns
- exceptions

---

## 3. TAURI COMMAND 정리 (Rust)

TASK
Normalize Tauri commands

SCOPE
- include: src-tauri/src/**/*.rs

RULES
- enforce snake_case naming
- return Result types
- no frontend logic duplication
- keep DB access in Rust only

VALIDATION
- cargo check passes

OUTPUT
- summary only
- commands updated
- inconsistencies

---

## 4. DB 접근 경계 강제

TASK
Enforce DB access boundary

SCOPE
- include: src/**/*

RULES
- DB access only in Rust
- no SQL or DB logic in frontend
- store communicates via invoke()

VALIDATION
- no DB usage in frontend

OUTPUT
- summary only
- violations fixed
- remaining issues

---

## 5. EXCEL 처리 분리

TASK
Separate Excel logic from UI

SCOPE
- include: src/**/*

RULES
- ExcelJS logic moved to util/service layer
- components only pass data
- no parsing logic in UI

VALIDATION
- import/export works correctly

OUTPUT
- summary only
- moved functions
- new modules

---

## 6. LARGE REFACTOR (INTERACTIVE MODE)

TASK
Refactor module to consistent architecture

SCOPE
- include: src/sections/<target>/**/*

RULES
- follow existing project patterns
- do not change behavior
- extract reusable logic

VALIDATION
- build passes
- no regression

OUTPUT
- summary only
- files changed
- major changes
- risks

---

## 7. TEST GENERATION

TASK
Generate unit tests

SCOPE
- include: src/**/*.ts

RULES
- cover public functions
- include success + failure cases
- mock external dependencies

VALIDATION
- tests must pass

OUTPUT
- summary only
- test files created
- coverage estimate

---

## 8. ERROR HANDLING STANDARDIZATION

TASK
Standardize error handling

SCOPE
- include: src/**/*

RULES
- use explicit try/catch
- no silent failures
- consistent error propagation

VALIDATION
- no uncaught errors

OUTPUT
- summary only
- updated files
- edge cases

---

## 9. UI CLEANUP (Vue)

TASK
Improve UI readability and structure

SCOPE
- include: src/components/**/*.vue

RULES
- improve readability (spacing, structure)
- maintain existing design
- no logic changes

VALIDATION
- UI renders correctly

OUTPUT
- summary only
- components updated

---

## 10. SAFE REFACTOR (LOW RISK)

TASK
Apply safe refactoring without behavior change

SCOPE
- include: <target>

RULES
- no logic change
- only structural improvements
- keep API unchanged

VALIDATION
- build + test pass

OUTPUT
- summary only
- files changed