# ci-cd (delta)

## ADDED Requirements

### Requirement: Spec validation gate

Whenever anything under `openspec/` changes, the change SHALL pass
`openspec validate --all --strict` before it is considered done (CLAUDE.md
Definition of done, "Specs" gate).

#### Scenario: Spec edit is validated

- GIVEN a branch that modifies a file under `openspec/`
- WHEN the Definition-of-done verify loop runs
- THEN `openspec validate --all --strict` reports zero failures
