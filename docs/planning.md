# Planning work

Durop organizes work as **events → release candidates → features and bugs**.
This adopts Maybraid's planning model, with Durop-specific acceptance criteria and
independently written templates. These are issue categories, not Git tags or a
claim that a release has shipped.

| Category | Title | Required information |
| --- | --- | --- |
| `event` | `Event: <name>` | Target date, intended demonstration, linked candidates, evidence owner. |
| `release-candidate` | `Release Candidate: <version or milestone>` | Linked events, included feature/bug issues, usable result, acceptance gates. |
| `feature` | One sentence describing the capability | User-visible outcome, candidate link, acceptance criteria. |
| `bug` | One sentence describing the defect | Expected/actual behavior, reproduction, candidate link, regression check. |
| `proposal` | One sentence describing the decision | Constraints, alternatives, evidence and a link to the resulting decision. |

Use the [issue forms](../.github/ISSUE_TEMPLATE/) when opening work. An unscheduled
item may name a pending parent and its owner; link the parent before including the
item in a candidate. Add reciprocal links so events expose their candidates and
candidates expose their implementation work. GitHub sub-issues may reinforce these
links, but a hierarchy visible in issue bodies is the baseline.

Every issue has exactly one priority: `priority:low`, `priority:medium`,
`priority:high` or `priority:urgent`. The form records the requested priority;
the maintainer applies the matching label during triage. Optional `docs`,
`delivery` and `validation` labels describe the kind of work without replacing its
category or priority. The README tables show high/urgent work; lower-priority
items remain available in the full issue list.

A candidate is ready when its stated gates pass and linked blockers are resolved.
For Durop, distinguish automated checks from actual desktop, audio, recording and
two-Mac evidence. An event is complete when the planned demonstration or acceptance
has occurred and its evidence is linked. Do not close device acceptance work on
the strength of unit tests alone.

Label names and descriptions live in [`.github/labels.json`](../.github/labels.json).
The **Issue label definitions** workflow creates or updates them when that file
lands on `main`, or when manually dispatched. It does not delete other labels,
relabel existing issues, create milestones or publish releases. Templates and
labels become the default repository workflow when this branch is merged.

The model was informed by Maybraid revision
`0d25b9777709253f572d22f74e227c630abf7353`; no implementation or template text was
copied from that source-available project.
