## Plan Generated: Blueprint-Driven Detailed Plan

### TL;DR
> Summary: Convert the BLUEPRINT.md blueprint into a formal, executable plan. Create missing memory docs scaffolds and translate blueprint sections into explicit tasks with acceptance criteria. All decisions are resolved within this plan to support ZERO-JUDGMENT execution by agents.
> Deliverables: A single plan document under .sisyphus/plans/blueprint-plan.md containing tasks, wave-based execution, QA scenarios, and evidence artifacts.
> Effort: Large
> Parallel: YES
> Critical Path: Extract blueprint sections -> Define memory scaffolds -> Populate detailed tasks -> Metis/Oracle verification

## Context
### Original Request
- Based on BLUEPRINT.md, generate a detailed, decision-complete execution plan for advancing Pawlos planning work.
- The blueprint is located at G:\pawlos\BLUEPRINT.md (root, uppercase). The lowercase blueprint.md does not appear to exist in this workspace.

### Interview Summary (from silent exploration)
- The blueprint outlines Architecture Overview, Memory System, Onboarding, and multi-provider support. It references memory artifacts MEMORY.md, USER.md, SOUL.md that are not present in the repository.
- No runtime plan yet; this document aims to convert blueprint structure into actionable tasks with acceptance criteria and QA scenarios.

### Metis Review (Guardrails)
- To be incorporated in Phase 4; current draft includes placeholders for guardrails and decision-needed items.

## Work Objectives
### Core Objective
- Produce a decision-complete work plan that translates blueprint content into explicit tasks for agents, including creation of missing memory docs and a map of execution waves.
### Deliverables
- .sisyphus/drafts/blueprint-plan.md (draft)
- .sisyphus/plans/blueprint-plan.md (final plan)
- .sisyphus/evidence/task-<N>-blueprint-plan.* as QA evidence (to be populated during execution)

### Definition of Done (verifiable)
- All blueprint sections are represented as concrete tasks with acceptance criteria.
- All tasks include QA scenarios with explicit selectors and data requirements.
- Plan contains a clear dependency/wave structure and an agent-dispatch summary.
- The plan is stored in the designated path and ready for execution via /start-work.

## Work Objectives (Detailed)
- Core Objective: Translate BLUEPRINT.md into executable tasks for the Pawlos planning workflow.
- Deliverables: Completed blueprint-driven plan with memory scaffolds and QA artifacts.
- Definition of Done: All tasks have concrete acceptance criteria and QA scenarios; no open decisions remain at this planning level.
- Must Have: 1) Memory scaffolds 2) Section-to-task mappings 3) Wave-based execution plan 4) Verification approach.
- Must NOT Have: Direct code changes; actions limited to plan generation and documentation.

## Verification Strategy
- Verification will be agent-executed (no human intervention). Each task must include a QA scenario with concrete steps and expected outcomes.
- Evidence artifacts to be produced at .sisyphus/evidence/task-*-blueprint-plan.*

## Execution Strategy
- Wave 1: Parse BLUEPRINT.md and identify top-level sections.
- Wave 2: Create scaffolds for MEMORY.md, USER.md, SOUL.md; map blueprint sections to draft tasks.
- Wave 3: Populate detailed tasks per blueprint section (Architecture, Memory, Tools, Lifecycle).
- Wave 4: Run a Metis/Oracle guardrails check; address any gaps.

### Parallelization
- Target: 5-8 tasks per wave; parallelize where there are no dependencies.

### Dependency Matrix
- blueprint extraction -> memory scaffold creation -> task detailing per section -> verification passes.

### Agent Dispatch Summary
- Wave 1: explore/librarian to extract blueprint structure
- Wave 2: planner to draft task mapping and scaffolds
- Wave 3: subagents to fill in detailed tasks and acceptance criteria
- Wave 4: Oracle/Metis for guardrail review

## TODOs (atomic, with acceptance criteria)
- [ ] 1. Extract blueprint sections from BLUEPRINT.md and produce a mapping to plan sections
- [ ] 2. Create MEMORY.md, USER.md, SOUL.md scaffolds (placeholders) and reference them in the plan
- [ ] 3. Draft detailed tasks for each blueprint section (Architecture, Memory, Tools, etc.) with acceptance criteria
- [ ] 4. Create draft (.sisyphus/drafts/blueprint-plan.md) summarizing decisions and questions
- [ ] 5. Schedule Metis review and integrate guardrails
- [ ] 6. Produce final plan at .sisyphus/plans/blueprint-plan.md
- [ ] 7. Present plan and decision options to user (Start Work vs High-Accuracy Review)

## Final Verification Wave (MANDATORY — after ALL tasks)
- F1. Plan Compliance Audit — oracle
- F2. Code Quality Review — unspecified-high
- F3. Real Manual QA — unspecified-high
- F4. Scope Fidelity Check — deep

## Commit Strategy
- Commit only plan artifacts; no code changes in this phase.
