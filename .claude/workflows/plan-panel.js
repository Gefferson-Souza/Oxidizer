export const meta = {
  name: 'tyrus-plan-panel',
  description: 'P3 large-tier planning: 3 concurrent plan candidates + adversarial judge',
  phases: [
    { title: 'Plan', detail: '3 independent planners (MVP-first, risk-first, smallest-blast-radius)' },
    { title: 'Judge', detail: 'adversarial synthesis into one plan' },
  ],
}

// args: { issue: number, title: string, acceptance: string, surface: string }
const unit = args

const CONSTRAINTS = `
BINDING CONSTRAINTS (Tyrus — restate-don't-assume, F8):
- Normative spec: docs/standards/POWER_OF_TEN.md (R1-R14) + docs/standards/DEVELOPMENT_FLOW.md (F1-F10).
- RULE ZERO: any codegen/analyzer behavior change needs a semantic equivalence test that is RED first
  (tests/src/equivalence/, valid TypeScript through assert_output_equivalent). Observed output defines done.
- Architecture: two layers only — generic AST handler per node TYPE + semantic registry when a NAME matters
  (tyrus_decorator_kinds / decorators::shared_registry / stdlib). NEVER scattered match-arm name dispatch.
- Code: quote! only (no string-concat codegen), no unwrap/expect/panic/todo, functions <= 50 lines,
  files <= 400, pub(crate) for internals, #![forbid(unsafe_code)].
- Scope: ONLY what issue #${unit.issue} states. Reuse before creating: check CLAUDE.md crate map,
  tyrus_common::util, decorators/, stdlib/. Flag any R10/F9 ADR trigger explicitly.
- Output in Portuguese. Plans are READ-ONLY analysis — do not edit files.

WORK UNIT: issue #${unit.issue} — ${unit.title}
Acceptance: ${unit.acceptance}
Surface: ${unit.surface}
`

const PLAN_SCHEMA = {
  type: 'object',
  additionalProperties: false,
  properties: {
    approach: { type: 'string' },
    slices: {
      type: 'array',
      items: {
        type: 'object',
        additionalProperties: false,
        properties: {
          description: { type: 'string' },
          files: { type: 'string' },
          acceptance: { type: 'string' },
          risk: { type: 'string' },
        },
        required: ['description', 'files', 'acceptance', 'risk'],
      },
    },
    adr_trigger: { type: 'string' },
    reuse_checked: { type: 'string' },
    open_questions: { type: 'array', items: { type: 'string' } },
  },
  required: ['approach', 'slices', 'adr_trigger', 'reuse_checked', 'open_questions'],
}

const lenses = [
  { key: 'mvp', prompt: 'LENS: MVP-FIRST. Smallest change that satisfies the acceptance criterion; defer everything deferrable (as new issues, named).' },
  { key: 'risk', prompt: 'LENS: RISK-FIRST. Identify the riskiest unknown (equivalence edge case, registry interaction, analyzer coupling) and order slices to retire it first.' },
  { key: 'blast', prompt: 'LENS: SMALLEST-BLAST-RADIUS. Minimize files/crates touched; prefer registry entries and existing seams over new structure.' },
]

const plans = (await parallel(lenses.map((l) => () =>
  agent(
    CONSTRAINTS + '\n' + l.prompt + '\nRead the real code before proposing anything (cite file:line). Produce a sliced plan.',
    { label: 'plan:' + l.key, phase: 'Plan', schema: PLAN_SCHEMA }
  )
))).filter(Boolean)

const judged = await agent(
  CONSTRAINTS +
  '\nYou are the adversarial JUDGE. Three plan candidates (JSON):\n' + JSON.stringify(plans, null, 1) +
  '\nTry to BREAK each plan: scope leaks, hidden large slices, red-line risk, missed reuse, missing RED test,' +
  ' unflagged ADR trigger. Then synthesize THE plan (steal the best slices, fix the holes).' +
  ' Verify every file:line claim against the repo before keeping it.',
  {
    label: 'judge', phase: 'Judge',
    schema: {
      type: 'object',
      additionalProperties: false,
      properties: {
        verdict: { type: 'string', enum: ['APPROVED', 'APPROVED_WITH_CAVEATS', 'REPLAN'] },
        caveats: { type: 'array', items: { type: 'string' } },
        final_plan: PLAN_SCHEMA,
        rejected_ideas: { type: 'array', items: { type: 'string' } },
      },
      required: ['verdict', 'caveats', 'final_plan', 'rejected_ideas'],
    },
  }
)

return { unit, candidates: plans.length, judged }
