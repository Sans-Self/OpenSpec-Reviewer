## MODIFIED Requirements

### Requirement: New members can read the full history they are admitted to

Admitting a member SHALL grant them wrapped access to the historical
group keys, not only the current one, so documents written under prior
rotations are readable by them per document-crypto's rotation-selected
reads. Admission after many rotations is not a degraded membership.

#### Scenario: post-rotation joiner reads a pre-rotation document

- **WHEN** a member is added at rotation n+3 and opens a document whose
  wrap targets rotation n
- **THEN** their keyring wraps grant the rotation-n key and the read
  succeeds
