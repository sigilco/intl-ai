# Security Policy

## Supported versions

Only the latest minor release line receives security fixes.

| Version | Supported |
| ------- | --------- |
| 0.7.x   | yes       |
| < 0.7   | no        |

## Reporting a vulnerability

Report vulnerabilities privately through GitHub: use "Report a vulnerability"
on this repository's Security tab (private vulnerability reporting). Do not
open a public issue.

Reports are acknowledged as soon as possible. If a report is confirmed, a fix
is prepared on a private branch and shipped in the next patch release, with
credit in the release notes if you want it.

## Trust boundaries

- `intl-ai` runs external programs only when the config explicitly asks:
  `kind = "command"` providers, `[[checks]] kind = "exec"` checks, and
  `[[formats]]` exec format plugins. Exec checks and format plugins are only
  honored from the project-root config, never from vendored or nested
  configs, so pulling a dependency cannot silently run its code.
- Secrets come from `${env:VAR}` interpolation or the process environment;
  they are never written to locale files, the lockfile, or reports.
