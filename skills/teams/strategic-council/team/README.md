# Strategic Council

> Phase: **Teams**

Council of business personas — consult 5 diverse strategic perspectives on product, design, data, growth, and operations decisions.

## Council Members

| Role | Bias | Model |
|------|------|-------|
| Product Lead | User value and business impact | opus |
| Design Lead | UX and interaction patterns | opus |
| Data/Analytics Lead | Metrics and measurement | opus |
| Growth Lead | Acquisition and retention | opus |
| Operations Lead | Team capacity and process | opus |

## How it works

The strategic council facilitates consultation on business and product decisions, not execution. You provide a business or product question to evaluate. The facilitator spawns all 5 council members in parallel as background agents. Each member analyzes the decision from their unique perspective and bias. After all members respond, the facilitator synthesizes their feedback into a decision matrix (Perspective | Concern | Recommendation) and highlights consensus vs. disagreement.

## When to use

- Product decisions (feature prioritization, roadmap planning)
- UX/design direction (new user flows, interface changes)
- Growth strategies (onboarding optimization, retention)
- When you want diverse business perspectives before deciding

## When NOT to use

- Simple implementation questions (ask directly)
- When you've already decided and just need execution
- Technical architecture decisions (use technical-council)

## Usage

```bash
/strategic-council Should we add a free tier or keep pricing simple?
```

## Install

```bash
forja install teams/strategic-council/team
```
