# Issue tracker: GitHub

Issues and specs for this repo live in GitHub Issues and are tracked in the dedicated `imajha/ui` GitHub Project.

- Repository: <https://github.com/devaryakjha/ui>
- Project: <https://github.com/users/devaryakjha/projects/5>
- Use the `gh` CLI for all operations.
- Add each created issue to the project with `gh project item-add 5 --owner devaryakjha --url <issue-url>`.

## Conventions

- Create: `gh issue create --title "..." --body "..."`
- Read: `gh issue view <number> --comments`
- List: `gh issue list --state open --json number,title,body,labels,comments`
- Comment: `gh issue comment <number> --body "..."`
- Label: `gh issue edit <number> --add-label "..."` or `--remove-label "..."`
- Close: `gh issue close <number> --comment "..."`
- Infer the repository from the current Git remote.

## Pull requests as a triage surface

**PRs as a request surface: no.**

## Skill operations

- “Publish to the issue tracker” means create a GitHub issue and add it to the project.
- “Fetch the relevant ticket” means run `gh issue view <number> --comments`.
