# Execution

Xerxes grows bottom-up: build a small game, solve its real problems, and move only proven, repeated pieces into the engine (the exception, the tools every game needs, is described in PROGRESS.md, Phase 3). One stage at a time, in the order PROGRESS.md gives.

Stage loop: mark a stage `in_progress` in PROGRESS.md, implement, run `__ctrl__\xerxes-ctrl.bat test all` (or `test engine` / `test backend` for the touched side), then mark it `done`.

## The daily rhythm

The pace is one small step a day, and consistency matters more than speed. Four habits keep that cheap:

1. **A visible next step.** PROGRESS.md starts with one line, **Next step**, saying exactly what the next small step is, so a day begins in a minute. Whoever finishes a step (a person or an AI session) rewrites that line before stopping.
2. **Every day ends green.** Finish with `test all` passing, so tomorrow never starts on a broken build. If a step cannot finish green, shrink it until it can, or revert it, and write why in the log. Keep disk space in mind: Bevy builds are tens of GB, and `cleanup` clears them (a full drive broke a Windows link once).
3. **Thin and working over wide.** Take a stage as a vertical slice: one mode playable offline, then online, before the next scene; one tool used by one scene before it gets a second. A slice that runs on screen is a better day than three half-built things.
4. **A short dated log.** `LOG.md` gets a few lines per day: what was done, what was learned, what is next. It is for the days you are away, not for anyone else.

A step is "small" when it fits in one sitting and leaves something you can run or test.
