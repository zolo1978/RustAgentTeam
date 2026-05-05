# Incident Response Protocol

| Severity | Response SLA | Communication |
|----------|-------------|---------------|
| Critical | 1 hour — patch, hotfix release | Incident channel + email to all users |
| High | 24 hours — patch branch | Incident channel, GH advisory draft |
| Medium | 7 days — next release cycle | GH issue + changelog entry |
| Low | 30 days — backlog | GH issue tracking |

## Response Checklist

1. Confirm and triage — assign Finding ID, CVSS score.
2. Contain — disable feature flag, revoke token, block network path.
3. Fix — write patch with regression test.
4. Verify — re-run cargo audit + fuzzing + manual test.
5. Release — signed hotfix, update changelog, publish advisory.
6. Post-mortem — root cause, timeline, prevention measures.
