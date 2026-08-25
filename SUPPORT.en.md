# Getting help

> 🇫🇷 [Version française](SUPPORT.md)

## First: is it the app, or the portal?

That distinction settles most requests. BetterAimaira reads your portal. It does
not drive it, and it has no access to your record.

**The project can do nothing about:** a wrong or missing grade, an absence
recorded badly, a timetable your school has not published yet, a forgotten
password, a locked account, a missing administrative document. All of that lives
in the portal. It is settled with your school's administration, and with nobody
else.

**The project can do something about:** the app refusing to sign in while the
portal works in a browser, a wrong reading while the portal reads right, a
crash, an update that never arrives, a school missing from the list.

## Before opening an issue

1. The [site FAQ](https://betteraimaira.montfrond.work/en/#faq) answers the
   questions that come up most.
2. The [compatibility page](https://betteraimaira.montfrond.work/en/schools)
   says whether your school is known and which portal address it uses.
3. The [existing issues](https://github.com/fr-timothe/BetterAimaira/issues?q=is%3Aissue)
   — open and closed both; yours may already be there.

## Opening an issue

[github.com/fr-timothe/BetterAimaira/issues](https://github.com/fr-timothe/BetterAimaira/issues)

What to put in:

- operating system and version;
- app version and the channel it follows (`stable` or `beta`);
- the error code the interface showed, verbatim;
- what you expected, and what happened;
- your school, if it helps — optional.

**What never goes in.** An issue is public, readable by anyone, and indexed by
search engines. So: no password, no cookie, no token, no screenshot showing your
name, your grades, your absences or other students, no PDF from the portal. If a
screenshot is needed, mask everything that identifies anyone.

## Missing school, or a wrong portal address

Open an issue. The directory's rules are in
[assets/schools/README.md](assets/schools/README.md): an address is never
written on a guess, it is confirmed first. If you know your portal's exact
address, say it — that is the part that is usually missing.

A school that would rather not appear in the list can ask to be removed the same
way: see [NOTICE.md](NOTICE.md).

## Security vulnerability

**Do not open a public issue.** Use the repository's `Security` tab, `Report a
vulnerability`, which opens a private thread. If that button is not available,
open an issue saying only that you have a security report and need a private
channel — no detail, no proof of concept.

The scope is BetterAimaira itself: the code in this repository and the artifacts
it publishes. Vulnerabilities in Aimaira portals are not this project's to
handle, and probing a portal you hold no account on is not security research.

## What to expect

A project run by one person, on their own time. Answers come when they come, and
nothing here promises a turnaround. A precise, reproducible issue carrying no
personal data is handled far faster than a vague report.

To contribute rather than report: [CONTRIBUTING.en.md](CONTRIBUTING.en.md).
