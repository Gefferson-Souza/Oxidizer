export const meta = {
  name: 'tyrus-adversarial-audit',
  description: 'P4.5/P5 panel: multi-dimension adversarial audit of a Tyrus diff + synthesis',
  phases: [
    { title: 'Audit', detail: '4 independent auditors (equivalence, tests, rules, adversarial)' },
    { title: 'Synthesis', detail: 'skeptic consolidation + verdict' },
  ],
}

// args: { diffFile: string (absolute path to a git diff), issue: number, acceptance: string }
const { diffFile, issue, acceptance } = args

const CONTEXT = `
You audit a diff of Tyrus, a TypeScript-to-Rust transpiler (Cargo workspace, crates/ + tests member).
Diff under audit: read it from ${diffFile} — then ALWAYS read the CURRENT repo code around every
finding; the diff hunk alone is not ground truth. Issue: #${issue}. Acceptance: ${acceptance}

BINDING RULES (verify against, cite rule ids): docs/standards/POWER_OF_TEN.md (R1-R14),
docs/standards/DEVELOPMENT_FLOW.md (F1-F10). Key CRITICALs: R5 equivalence-test density,
R6 no panics/unwrap/indexing, R7 quote!-only codegen, R8 two-layer architecture (registry,
never scattered name matches), F2 test was RED first, F6 observed output.

Method: for each candidate finding, actively try to refute it first; report only survivors as
CONFIRMED (verified in code) or PLAUSIBLE, with file:line and a concrete failure scenario
(input -> wrong output). You may run read-only commands (cargo nextest run <filter>, cargo clippy)
to observe instead of inferring. NEVER edit files. Respond in Portuguese.
`

const SCHEMA = {
  type: 'object',
  additionalProperties: false,
  properties: {
    dimension: { type: 'string' },
    verdict: { type: 'string', enum: ['PASS', 'CONCERN', 'FAIL'] },
    summary: { type: 'string' },
    findings: {
      type: 'array',
      items: {
        type: 'object',
        additionalProperties: false,
        properties: {
          location: { type: 'string' },
          observation: { type: 'string' },
          severity: { type: 'string', enum: ['LOW', 'MEDIUM', 'HIGH', 'CRITICAL'] },
          confidence: { type: 'string', enum: ['CONFIRMED', 'PLAUSIBLE'] },
          evidence: { type: 'string' },
        },
        required: ['location', 'observation', 'severity', 'confidence', 'evidence'],
      },
    },
  },
  required: ['dimension', 'verdict', 'summary', 'findings'],
}

const dims = [
  { key: 'equivalence', prompt: `DIMENSION: SEMANTIC EQUIVALENCE (RULE ZERO). Does every behavior change in the diff have an equivalence/reproduction test? Is the TS snippet valid TypeScript a Node runtime accepts? Would the test fail if the key generated line were broken (judge the assertion, and run the mutation yourself on a scratch copy ONLY if read-only checks are inconclusive — otherwise reason from the assertion text)? Look for outputs asserted loosely (contains vs exact stdout).` },
  { key: 'tests', prompt: `DIMENSION: TEST QUALITY. Do the tests exercise the ACCEPTANCE CRITERION of issue #${issue}, or only the type system/framework plumbing? Isolation (no shared state/ports), snapshot diffs intentional, anti-regression for the bug class covered, test names honest about what they assert.` },
  { key: 'rules', prompt: `DIMENSION: POWER OF TEN COMPLIANCE. Hunt violations the lints cannot see: R7 (any string assembly feeding codegen), R8 (name-keyed logic outside the registries), R6 escape hatches (#[expect] with weak reasons, test-only allows leaking into prod paths), R4 (functions grown past 50 lines), and doc claims the code does not back (ADR 0013 honest-enforcement).` },
  { key: 'adversarial', prompt: `DIMENSION: ADVERSARIAL. Try to BREAK the change: semantic shifts hiding in mechanical rewrites, fallbacks silently swallowing what used to be noisy, signature changes with missed call sites, edge inputs (empty program, unicode identifiers, nested constructs) that the happy path misses. Propose 2-3 concrete TS inputs that would expose a bug, and reason through what the generated Rust would do.` },
]

const audits = (await parallel(dims.map((d) => () =>
  agent(CONTEXT + '\n\n' + d.prompt, { label: 'audit:' + d.key, phase: 'Audit', schema: SCHEMA })
))).filter(Boolean)

const synth = await agent(
  CONTEXT + '\nYou are the SKEPTIC SYNTHESIZER. The 4 audit reports (JSON):\n' +
  JSON.stringify(audits, null, 1) +
  '\nDeduplicate, then re-verify every CRITICAL/HIGH yourself against the repo before sustaining it.' +
  ' Downgrade anything the code refutes. Output the consolidated verdict: APPROVED only if zero' +
  ' sustained CRITICAL/HIGH.',
  {
    label: 'synthesis', phase: 'Synthesis',
    schema: {
      type: 'object',
      additionalProperties: false,
      properties: {
        overall_verdict: { type: 'string', enum: ['APPROVED', 'FIX_REQUIRED'] },
        sustained: {
          type: 'array',
          items: {
            type: 'object',
            additionalProperties: false,
            properties: {
              title: { type: 'string' },
              severity: { type: 'string' },
              location: { type: 'string' },
              action: { type: 'string' },
            },
            required: ['title', 'severity', 'location', 'action'],
          },
        },
        refuted: { type: 'array', items: { type: 'string' } },
      },
      required: ['overall_verdict', 'sustained', 'refuted'],
    },
  }
)

return { issue, audits, synth }
