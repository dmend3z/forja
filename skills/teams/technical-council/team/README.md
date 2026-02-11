# Technical Council

> Phase: **Teams**

Council of engineering personas — consult 5 diverse technical perspectives on architecture, infrastructure, security, quality, and performance decisions.

## Council Members

| Role | Bias | Model |
|------|------|-------|
| Principal Engineer | Simplicity and long-term maintenance | opus |
| Platform Engineer | Operational burden and infrastructure costs | opus |
| Security Engineer | Threat modeling and compliance | opus |
| QA Lead | Testability and edge cases | opus |
| Performance Engineer | Latency, throughput, bottlenecks | opus |

## How it works

The technical council facilitates consultation on engineering decisions, not execution. You provide a technical question or decision to evaluate. The facilitator spawns all 5 council members in parallel as background agents. Each member analyzes the decision from their unique perspective and bias. After all members respond, the facilitator synthesizes their feedback into a decision matrix (Perspective | Concern | Recommendation) and highlights consensus vs. disagreement.

## When to use

- Architecture decisions (monolith vs. microservices, database choice)
- Infrastructure changes (deployment strategy, observability)
- Technical debt prioritization
- When you want diverse expert perspectives before deciding

## When NOT to use

- Simple implementation questions (ask directly)
- When you've already decided and just need execution
- Product or business decisions (use strategic-council)

## Usage

```bash
/technical-council Should we migrate from REST to GraphQL?
```

## Install

```bash
forja install teams/technical-council/team
```
